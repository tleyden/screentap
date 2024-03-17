extern crate screen_ocr_swift_rs;

use std::time::{Instant, Duration};
use serde::Serialize;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use reqwest::blocking::Response;
use tauri::Manager;
use std::fmt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str::FromStr;
use base64::engine::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64;
use std::error::Error;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use ollama_rs::{
    generation::completion::request::GenerationRequest,
    generation::images::Image,
    Ollama,
};
use tokio::runtime;
use rusqlite::Result;
use image_hasher::{HasherConfig, ImageHash};
use event::FocusGuardCallbackEvent;
use result::{FocusGuardCallbackResult, SkipVisionModelReason};

mod utils;
pub mod handlers;

pub mod config;
pub mod event;
pub mod result;


// Create an enum with three possible values: openai, llamafile, and ollama
#[allow(dead_code)]
#[derive(PartialEq)]
#[derive(Clone)]
pub enum LlavaBackendType {

    // This works
    OpenAI,

    // This doesn't work due to a limitation with the LlamaFile + Llava JSON API
    // that cannot handle images. 
    LlamaFile,

    // Fork off a process and call LlamaFile's command line interface would work
    // to do inference on Llava.  This is working great.
    LlamaFileSubprocess,

    // This doesn't work because yet because the version of Llava ignores the 
    // instructions and doesn't return a single number.  Also sometimes it
    // doesn't even return any score at all.
    Ollama,
}



impl fmt::Display for LlavaBackendType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LlavaBackendType::OpenAI => write!(f, "OpenAI"),
            LlavaBackendType::LlamaFile => write!(f, "LlamaFile"),
            LlavaBackendType::LlamaFileSubprocess => write!(f, "LlamaFileSubprocess"),
            LlavaBackendType::Ollama => write!(f, "Ollama"),
        }
    }
}

impl FromStr for LlavaBackendType {
    type Err = ();

    fn from_str(input: &str) -> Result<LlavaBackendType, Self::Err> {
        match input {
            "OpenAI" => Ok(LlavaBackendType::OpenAI),
            "LlamaFile" => Ok(LlavaBackendType::LlamaFile),
            "LlamaFileSubprocess" => Ok(LlavaBackendType::LlamaFileSubprocess),
            "Ollama" => Ok(LlavaBackendType::Ollama),
            _ => Err(()),
        }
    }
}

#[derive(PartialEq)]
#[derive(Clone)]
enum FocusGuardState {
    Idle,
    Primed,
}

impl fmt::Display for FocusGuardState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FocusGuardState::Idle => write!(f, "IDLE"),
            FocusGuardState::Primed => write!(f, "PRIMED"),
        }
    }
    
}

#[derive(Clone)]
pub struct FocusGuard {
    pub job_title: String,
    pub job_role: String,
    pub openai_api_key: String,

    // How long to delay before showing next distraction alert (eg, 30 mins)
    duration_between_alerts: Duration,

    // Track the last time a distraction alert was shown
    last_distraction_alert_time: Instant,

    // The backend to use for the LLaVA model
    llava_backend: LlavaBackendType,

    // The threshold to be considered in "flow state"
    productivity_score_threshold: i32,

    // How much to scale down the raw screenshot before sending to the vision model.
    // This must be a number between 0.1 and 1.0.
    image_resize_scale: f32,

    // The path to the app data directory in order to find plugin assets like
    // the Llamafile binary
    app_data_dir: PathBuf,

    // The path to the screentap database, where focusguard will store its own tables
    screentap_db_path: PathBuf,

    // Amorphous dev mode flag to speed up dev
    dev_mode: bool,

    // The state used to determine when to invoke the vision model
    state: FocusGuardState,

    // The previous perceptual hash of the image
    previous_phash_opt: Option<ImageHash>,

}

impl FocusGuard {

    fn calculate_perceptual_hash(png_data: &[u8]) -> ImageHash {

        // Get the current time
        let now = Instant::now();

        let hasher_config = HasherConfig::new().hash_size(32, 32).preproc_dct();
        let hasher = hasher_config.to_hasher();
        let img = image::load_from_memory(png_data).unwrap();
        let hashed_img = hasher.hash_image(&img);

        // calculate the time it took to hash the image
        let time_to_hash = now.elapsed();
        println!("Time to calculate perceptual hash: {:?}", time_to_hash);

        hashed_img

    }

    pub fn get_db_conn(screentap_db_path: &PathBuf) -> rusqlite::Connection {
        rusqlite::Connection::open(screentap_db_path).unwrap()
    }

    fn create_table_if_doesnt_exist(screentap_db_path: &PathBuf) -> Result<()> {

        // Create the focusguard database table if it doesn't exist
        let conn = FocusGuard::get_db_conn(screentap_db_path);

        // Create a table with the desired columns
        conn.execute(
            "CREATE TABLE IF NOT EXISTS focusguard_distraction_alerts (
                    id INTEGER PRIMARY KEY,
                    screenshot_id INTEGER,
                    user_rating INTEGER,
                    timestamp TIMESTAMP NOT NULL,
                    file_path TEXT NOT NULL,
                    job_title TEXT NOT NULL,
                    job_role TEXT NOT NULL
                )",
            [],
        )?;

        // TOOD: create the event log table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS focusguard_event_log (
                    id INTEGER PRIMARY KEY,
                    screenshot_id INTEGER,
                    invoked_vision_model INTEGER,
                    vision_model_success INTEGER,
                    vision_model_descriptor TEXT NOT NULL,
                    skip_vision_model_reason TEXT NOT NULL
                )",
            [],
        )?;


        Ok(())
    
    }

    pub fn new_from_config(app_data_dir: PathBuf, screentap_db_path: PathBuf) -> Option<FocusGuard> {

        // Register plugin - create a new focusguard struct
        let focus_guard_config = config::FocusGuardConfig::new(app_data_dir.as_path());
        let focus_guard = match focus_guard_config {
            
            Some(config) => {

                let llava_backend = match LlavaBackendType::from_str(&config.llava_backend) {
                    Ok(llava_backend) => llava_backend,
                    Err(_) => {
                        println!("Invalid LlavaBackendType: {}.  Not starting FocusGuard plugin.", config.llava_backend);
                        return None
                    }
                };

                if llava_backend == LlavaBackendType::OpenAI && config.openai_api_key.is_empty() {
                    println!("OpenAI API key is required for OpenAI backend");
                    return None
                };

                let duration_between_alerts = Duration::from_secs(config.duration_between_alerts_secs);
        
                // Initialize tracking vars so that it begins with an initial check
                let last_distraction_alert_time = Instant::now() - duration_between_alerts - Duration::from_secs(1);

                FocusGuard::create_table_if_doesnt_exist(&screentap_db_path).expect("Error creating focusguard tables");
        
                FocusGuard {
                    job_title: config.job_title,
                    job_role: config.job_role,
                    openai_api_key: config.openai_api_key,
                    duration_between_alerts,
                    last_distraction_alert_time,
                    llava_backend,
                    productivity_score_threshold: config.productivity_score_threshold,
                    image_resize_scale: config.image_resize_scale,
                    app_data_dir,
                    screentap_db_path,
                    dev_mode: config.dev_mode,
                    state: FocusGuardState::Idle,
                    previous_phash_opt: None,
                }

            },
            None => {
                println!("Unable to load FocusGuard config.  This plugin will not be enabled");
                return None
            }   
        };

        Some(focus_guard)

    }

    /**
     * If this is called twice in a row with the same frontmost_app or browser tab, it means the user is "lingering" on 
     * that app/tab rather than just in transit between apps.  It should invoke the vision model
     * 
     * This expects to be called back from the screentap event handler every 30s.  If it was called back more frequently
     * (eg, every 5s), it would have to also take timing into account.  But 30s is a good delay between state transitions
     * from IDLE -> PRIMED.
     */
    pub fn should_invoke_vision_model(&mut self, frontmost_app: &str, frontmost_browser_tab: &str, frontmost_app_or_tab_changed: bool) -> Option<SkipVisionModelReason>  {

        println!("FocusGuard checking if should_invoke_vision_model: frontmost_app: {} frontmost_browser_tab: {} frontmost_app_or_tab_changed: {} cur state: {}", frontmost_app, frontmost_browser_tab, frontmost_app_or_tab_changed, self.state);

        // Special handlers if the frontmost app is missing or the screentap app itself
        if frontmost_app == "missing value" || frontmost_app.starts_with("com.screentap-app") {  
            println!("FocusGuard or a missing value is the frontmost app, so not invoking vision model and resetting state to IDLE");
            self.state = FocusGuardState::Idle;
            return Some(SkipVisionModelReason::InvalidFrontmostApp);
        };

        match self.state {
            FocusGuardState::Primed => {
                // the state is primed, meaning we have already gotten out of the IDLE state and may be ready to invoke the vision model
                if !frontmost_app_or_tab_changed {
                    // The system is primed and the user is lingering in the same app or browser tab, 
                    // therefore we should invoke the vision model and reset the state to IDLE
                    println!("FocusGuard invoking vision model ...");
                    self.state = FocusGuardState::Idle;
                    None
                } else {
                    // The system is primed but the user has switched to a different app or browser tab,
                    // reset the state to IDLE and do not invoke the vision model
                    println!("FocusGuard not invoking vision model, and resetting state to IDLE");
                    self.state = FocusGuardState::Idle;
                    Some(SkipVisionModelReason::NotPrimed)
                }    
            },
            FocusGuardState::Idle => {
                if !frontmost_app_or_tab_changed {
                    // If the app hasn't changed, then it looks like the user is lingering in the same app or browser tab,
                    // so we want to go into the PRIMED state
                    println!("FocusGuard not invoking vision model, and going into PRIMED state");
                    self.state = FocusGuardState::Primed;
                }
                else {
                    // The app has changed so the user is still in transit between apps, stay in the IDLE state
                    println!("FocusGuard not invoking vision model, and staying in IDLE state");
                }
                Some(SkipVisionModelReason::NotPrimed)    
            },
        }
        
    }


    fn process_focus_guard_event(&mut self, cb_event: FocusGuardCallbackEvent) -> FocusGuardCallbackResult {

        let mut cb_result = FocusGuardCallbackResult::new();

        // Get the current time
        let mut now = Instant::now();

        // Check if we should invoke the vision model based on current frontmost app
        let should_skip_vision_model = self.should_invoke_vision_model(cb_event.frontmost_app, cb_event.frontmost_browser_tab, cb_event.frontmost_app_or_tab_changed);
        if let Some(reason) = should_skip_vision_model {
            cb_result.invoked_vision_model = false;
            cb_result.skip_vision_model_reason = Some(reason);
            return cb_result;
        }


        // Check if enough time elapsed since last distraction alert.  If not, short circuit the screen 
        // analysis to reduce expensive vision model calls.  NOTE: this short-circuit will interfere with 
        // analytics tracking, so it may need to be removed once that is implemented
        let elapsed_alert = now.duration_since(self.last_distraction_alert_time);
        let enough_time_elapsed_alert = elapsed_alert > self.duration_between_alerts;
        if !enough_time_elapsed_alert {
            println!("FocusGuard: not enough time elapsed {:?} since last distraction alert, not analyzing screenshot", elapsed_alert);
            cb_result.invoked_vision_model = false;
            cb_result.skip_vision_model_reason = Some(SkipVisionModelReason::NotEnoughTimeElapsedSinceAlert);
            return cb_result;
        }

        // If dev mode is enabled, don't invoke the vision model and short-circuit the processing
        if self.dev_mode {
            println!("FocusGuard: dev mode is enabled, not invoking vision model");
            println!("FocusGuard returning hardcoded productivity score");
            cb_result.invoked_vision_model = false;
            cb_result.skip_vision_model_reason = Some(SkipVisionModelReason::DevMode);
            return cb_result;
        }
        
        println!("FocusGuard: resizing image (can be slow) ..");
        now = Instant::now();

                // Resize the image before sending to the vision model
                let resize_img_result = FocusGuard::resize_image(
                    cb_event.png_data, 
                    self.image_resize_scale
                );

                // Get the resized png data
                let resized_png_data = match resize_img_result {
                    Some(resized_img) => resized_img,
                    None => {
                        println!("Error resizing image: see logs.  FocusGuard will not analyze this screenshot.");
                        cb_result.invoked_vision_model = false;
                        cb_result.skip_vision_model_reason = Some(SkipVisionModelReason::Error);
                        return cb_result
                    }
                };

        let time_to_resize = now.elapsed();
        println!("Resized image by {} to {} bytes: time_to_resize: {:?}", self.image_resize_scale, resized_png_data.len(), time_to_resize);

        // Is the perceptual hash delta above the threshold?  If not, short circuit the call to the vision model
        // for massive cost savings in tokens and/or compute budget. 
        let above_threshold = self.phash_delta_above_threshold(&resized_png_data, cb_event.png_image_path);
        if !above_threshold {
            cb_result.invoked_vision_model = false;
            cb_result.skip_vision_model_reason = Some(SkipVisionModelReason::PerceptualHashDuplicate);
            return cb_result
        }

        let prompt = self.create_prompt();

        let (productivity_score, raw_llm_result) = {
            // Invoke the actual vision model
            println!("FocusGuard analyzing image with {}.  Resizing image at png_image_path: {}", self.llava_backend, cb_event.png_image_path.display());

            now = Instant::now();

            let raw_result = match self.llava_backend {
                LlavaBackendType::OpenAI => self.invoke_openai_vision_model(&prompt, &resized_png_data),
                LlavaBackendType::Ollama => self.invoke_ollama_vision_model(&prompt, &resized_png_data),
                LlavaBackendType::LlamaFile => self.invoke_openai_vision_model(&prompt, &resized_png_data),
                LlavaBackendType::LlamaFileSubprocess => self.invoke_subprocess_vision_model(&prompt, cb_event.png_image_path),
            };

            cb_result.invoked_vision_model = true;
            cb_result.vision_model_descriptor = format!("{}", self.llava_backend);

            let time_to_infer = now.elapsed();
            println!("time_to_infer: {:?}", time_to_infer);

            match self.process_vision_model_result(&raw_result) { 
                Some(raw_result_i32) => {
                    (raw_result_i32, raw_result)
                },
                None => {
                    println!("FocusGuard could not parse raw result [{}] into number", raw_result);
                    cb_result.vision_model_success = false;
                    return cb_result;
                }
            }
    
        };

        // Record the productivity score in the database as this can be used for metrics tracking
        if productivity_score < self.productivity_score_threshold {
            println!("Productivity score {} is below threshold {} for png_image_path: {}", productivity_score, self.productivity_score_threshold, cb_event.png_image_path.display());

            self.show_productivity_alert(cb_event.app, productivity_score, &raw_llm_result, cb_event.png_image_path, cb_event.screenshot_id);
            self.last_distraction_alert_time = Instant::now();

        } else {
            println!("Woohoo!  Looks like you're working.  Score is: {} for png_image_path: {}", productivity_score, cb_event.png_image_path.display());
        }

        cb_result.vision_model_success = true;
        cb_result.productivity_score = productivity_score;
        cb_result.vision_model_response = Some(raw_llm_result);

        cb_result


    }


    #[allow(clippy::too_many_arguments)]
    pub fn handle_screentap_event(&mut self, app: &tauri::AppHandle, png_data: Vec<u8>, png_image_path: &Path, screenshot_id: i64, ocr_text: String, frontmost_app: &str, frontmost_browser_tab: &str, frontmost_app_or_tab_changed: bool) {

        let focusguard_event = FocusGuardCallbackEvent {
            app,
            png_data: &png_data,
            png_image_path,
            screenshot_id,
            ocr_text,
            frontmost_app,
            frontmost_browser_tab,
            frontmost_app_or_tab_changed
        };

        println!("Handling FocusGuard Event: {}", focusguard_event);

        let focus_guard_result = self.process_focus_guard_event(focusguard_event);

        println!("Focus guard result: {:?}", focus_guard_result);

        // self.record_result(focus_guard_result)



    }

    /**
     * Resize the image using the native Swift code
     */
    fn resize_image(png_data: &[u8], scale: f32) -> Option<Vec<u8>> {

        screen_ocr_swift_rs::resize_image(png_data, scale)

    }

    pub fn phash_delta_above_threshold(&mut self, png_data: &[u8], png_image_path: &Path) -> bool {

        println!("Calculating perceptual hash of image {} ...", png_image_path.display());
        let phash: ImageHash = FocusGuard::calculate_perceptual_hash(png_data);

        let result = match &self.previous_phash_opt {
            Some(previous_phash) => {
                let dist: u32 = phash.dist(previous_phash);
                let phash_threshold = 200;  // TODO: move to config.toml
                if dist < phash_threshold {  // TODO: tune this threshold
                    println!("phash delta is {}, which is below {} and not enough to warrant a new analysis", dist, phash_threshold);
                    false
                } else {
                    println!("phash delta is {}, which is above {} and enough to warrant a new analysis", dist, phash_threshold);
                    true
                }
            },
            None => {
                println!("phash: {}, but no previous phash to compare to.  New analysis needed.", phash.to_base64());
                true
            }
        };

        self.previous_phash_opt = Some(phash);
        
        result
    }

    fn show_productivity_alert(&self, app: &tauri::AppHandle, productivity_score: i32, raw_llm_result: &str, png_image_path: &Path, screenshot_id: i64) {

        println!("Showing productivity alert for score: {}", productivity_score);

        let window = app.get_window("focusguard");
        match window {
            Some(w) => {

                println!("Window exists, showing existing productivity alert window");

                // Window exists, so just bring it to the foreground
                w.show().unwrap();
                w.set_focus().unwrap();
                
                let event_name = "update-screenshot-event"; // The event name to emit
                let raw_llm_result_base64: String = BASE64.encode(raw_llm_result);

                let payload = serde_json::json!({
                    "screenshot_id": screenshot_id,
                    "productivity_score": productivity_score,
                    "raw_llm_result_base64": raw_llm_result_base64,
                    "png_image_path": png_image_path.to_str().unwrap(),
                    "job_title": self.job_title,
                    "job_role": self.job_role,
                    
                });

                // Emitting the event to the JavaScript running in the window
                if let Err(e) = w.emit(event_name, Some(payload)) {
                    eprintln!("Error emitting event: {}", e);
                }
            },
            None => {

                println!("Window does not exist, creating and showing new productivity alert window");

                // Use an init script approach when creating a new window, since sending an event
                // did not work in my testing.  Maybe it's not ready for events yet as some sort
                // of race condition?
                let init_script = get_init_script(
                    screenshot_id, 
                    productivity_score, 
                    raw_llm_result, 
                    png_image_path.to_str().unwrap(),
                    &self.job_title,
                    &self.job_role
                );

                // Create and show new window
                let _w = tauri::WindowBuilder::new(
                    app,
                    "focusguard",
                    tauri::WindowUrl::App("index_focusguard.html".into())
                ).initialization_script(&init_script).maximized(true).title("Focusguard").build().expect("failed to build window");


            }
        }   

    }
    
    fn process_vision_model_result(&self, raw_llm_response: &str) -> Option<i32> {

        println!("Raw LLM response: {}", raw_llm_response);

        match utils::find_first_number(raw_llm_response) {
            Some(raw_result_i32) => Some(raw_result_i32),
            None => {
                println!(r#"Error parsing raw LLM response "{}" into number"#, raw_llm_response);
                None
            }
        }

    }


    fn convert_png_data_to_base_64(&self, png_data: &Vec<u8>) -> String {
        BASE64.encode(png_data)
    }

    fn download_ollama_model(&self, model_name: &str) -> Result<(), Box<dyn Error>> {

        println!("Downloading Ollama model {}, this could take a while ...", model_name);

        // By default it will connect to localhost:11434
        let ollama = Ollama::default();

        // This is a hack needed to make this a blocking call rather than async
        let rt = runtime::Runtime::new().unwrap();

        // Pull model 
        let pull_model_future = ollama.pull_model(model_name.to_string(), true);

        // Block on the future
        let pull_model_result = rt.block_on(pull_model_future);

        match pull_model_result {
            Ok(_) => {
                println!("Successfully pulled Olllama model {}", model_name);
                Ok(())
            },
            Err(e) => {
                println!("Error pulling Ollama model: {}", e);
                Err(Box::new(e))
            }
        }


    }

    fn download_ollama_model_if_missing(&self, model_name: &str) -> Result<(), Box<dyn Error>> {

        // By default it will connect to localhost:11434
        let ollama = Ollama::default();

        // This is a hack needed to make this a blocking call rather than async
        let rt = runtime::Runtime::new().unwrap();

        // List local models to see if we already have the model
        let model_list_future = ollama.list_local_models();

        // Block on the future
        let model_list = rt.block_on(model_list_future);

        match model_list {
            Ok(model_list) => {

                // Does model_list contain a local model with name == model_name?
                for model in &model_list {
                    if model.name == model_name {
                        println!("Model {} already exists locally", model_name);
                        return Ok(())
                    }
                }

                println!("Model {} not found, downloading it now", model_name);
                let download_model_result = self.download_ollama_model(model_name);
                match download_model_result {
                    Ok(_) => Ok(()),
                    Err(e) => {
                        println!("Error downloading Ollama model: {}", e);
                        Err(e)
                    }
                }                
            },
            Err(e) => {
                println!("Error listing Ollama models: {}", e);
                Err(Box::new(e))
            }
        }

    }

    fn invoke_ollama_vision_model(&self, prompt: &str, png_data: &Vec<u8>) -> String {
                
        // By default it will connect to localhost:11434
        let ollama = Ollama::default();

        let model = "llava:7b-v1.6-mistral-q5_0".to_string();

        let download_model_result = self.download_ollama_model_if_missing(&model);
        match download_model_result {
            Ok(_) => println!("Downloaded model {} or it already existed", model),
            Err(e) => {
                println!("Error downloading Ollama model: {}", e);
                return "".to_string();
            }
        }

        // Getting the base64 string
        let base64_image = self.convert_png_data_to_base_64(png_data);

        let image = Image::from_base64(&base64_image);
        let req = GenerationRequest::new(
            model, 
            prompt.to_string()
        ).add_image(image);

        // This is a hack needed to make this a blocking call rather than async
        let rt = runtime::Runtime::new().unwrap();

        // Generate the response from the model
        let result_future = ollama.generate(req);

        // Block on the future
        let result = rt.block_on(result_future);

        match result {
            Ok(res) => {
                println!("Ollama result: {:?}", res.response);
                res.response.to_string()
            },
            Err(e) => {
                println!("Error invoking Ollama: {}.  Is Ollama running?", e);
                "".to_string()
            }
        }


    }


    fn download_llava_llamafile(&self, dest_file: &str) -> Result<(), Box<dyn Error>> {
        let url = "https://huggingface.co/jartine/llava-v1.5-7B-GGUF/resolve/main/llava-v1.5-7b-q4.llamafile?download=true";
        utils::download_file(url, dest_file)
    }

    fn invoke_subprocess_vision_model(&self, prompt: &str, png_image_path: &Path) -> String {

        let full_prompt = format!("### User: {}\n ### Assistant:", prompt);
        let llamafile_path = self.app_data_dir.join("plugins").join("focusguard").join("llava-v1.5-7b-q4.llamafile");
        if !llamafile_path.exists() {
            println!("Cannot find Llamafile at {}, downloading it now.  This may take several minutes/hours ..", llamafile_path.display());
            let dest_file: &str = llamafile_path.to_str().unwrap();
            match self.download_llava_llamafile(dest_file) {
                Ok(_) => { 
                    println!("Downloaded Llamafile to {}", dest_file);
                    let permissions = fs::Permissions::from_mode(0o755); // Equivalent to chmod +x
                    let _ = fs::set_permissions(dest_file, permissions);
                    println!("Set chmod +x permissions {}", dest_file);
                },
                Err(e) => {
                    println!("Error downloading Llamafile: {}", e);
                    return "".to_string();
                }
            }
        } 

        println!("Invoking LlamaFile subprocess ..");
        // sh ./llava-v1.5-7b-q4.llamafile -ngl 9999 --image ~/Desktop/2024_02_15_10_24_53.png -e -p '### User: On a scale of 1 to 10, how much does this screenshot indicate..'
        let output = Command::new(llamafile_path)
            .arg("-ngl")
            .arg("9999")
            .arg("--image")
            .arg(png_image_path)
            .arg("-e")
            .arg("-p")
            .arg(full_prompt)
            .output()
            .expect("Failed to execute command");

        // Assuming the command outputs something to stdout
        let result = if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
            println!("Raw output from llavafile: {}", &stdout);
            stdout
        } else {
            // If the command fails, e.g., non-zero exit status
            let stderr = String::from_utf8_lossy(&output.stderr);
            eprintln!("Error: {}", stderr);
            "".to_string()
        };

        result.trim().to_string()
        

    }

    fn invoke_openai_vision_model(&self, prompt: &str, png_data: &Vec<u8>) -> String {

        // Getting the base64 string
        let base64_image = self.convert_png_data_to_base_64(png_data);

        let client = reqwest::blocking::Client::new();

        let mut headers = HeaderMap::new();
        headers.insert(
            HeaderName::from_static("content-type"), 
            HeaderValue::from_static("application/json")
        );
        headers.insert(
            HeaderName::from_static("authorization"), 
            HeaderValue::from_str(&format!("Bearer {}", &self.openai_api_key)).unwrap()
        );

        let model_name = match self.llava_backend {
            LlavaBackendType::OpenAI => "gpt-4-vision-preview",
            LlavaBackendType::LlamaFile => "LLaMA_CPP",
            LlavaBackendType::LlamaFileSubprocess => "LLaMA_CPP",
            LlavaBackendType::Ollama => "TBD",
        };

        let payload = Payload {

            model: model_name.to_string(),
            messages: vec![
                Message {
                    role: "user".to_string(),
                    content: vec![
                        MessageContent {
                            content_type: "text".to_string(),
                            text: Some(prompt.to_string()),
                            image_url: None,
                        },
                        MessageContent {
                            content_type: "image_url".to_string(),
                            text: None,
                            image_url: Some(ImageUrl {
                                url: format!("data:image/jpeg;base64,{}", base64_image),
                            }),
                        },
                    ],
                },
            ],
            max_tokens: 4096,
        };

        println!("Invoking OpenAI API");

        let target_url = match self.llava_backend {
            LlavaBackendType::OpenAI => "https://api.openai.com/v1/chat/completions",
            LlavaBackendType::LlamaFile => "http://localhost:8080/v1/chat/completions",
            LlavaBackendType::LlamaFileSubprocess => panic!("Use a different method for LlamaFileSubprocess"),
            LlavaBackendType::Ollama => panic!("Use a different method for Ollama"),
        };

        let response_result = client.post(target_url)
            .headers(headers)
            .json(&payload)
            .send();

        let response = match response_result {
            Ok(response) => response,
            Err(e) => {
                println!("Error invoking vision model: {}", e);
                return "".to_string();
            }
        };
    
        self.extract_content_openai_response(response)
        
    }

    fn extract_content_openai_response(&self, response: Response) -> String {

        let response_json = match response.json::<serde_json::Value>() {
            Ok(response_json) => response_json,
            Err(e) => {
                println!("Error parsing response JSON: {}", e);
                return "".to_string();
            }
        };

        let choices = response_json["choices"].as_array();
        let first_choice = match choices {
            Some(choices) => {
                if choices.is_empty() {
                    println!("No choices in response.  Raw response: {}", response_json);
                    return "".to_string();
                }
                &choices[0]
            },
            None => {
                println!("No choices in response.  Raw response: {}", response_json);
                return "".to_string();
            }
        };

        let message_content = &first_choice["message"]["content"].as_str();

        message_content.unwrap_or("").to_string()

    }

    fn create_prompt(&self) -> String {
        let prompt = format!(r###"On a scale of 1 to 10, with 1 indicating the least amount
        of engagement and 10 indicating the most amount of engagement, how much does this screenshot indicate 
        a worker with job title of "{}" and job role of "{}" is currently engaged in work activities?  
        When analyzing the screenshots, please note that:
        * In many apps such as VS Code and Slack, the project name is often displayed in the top left corner in a slightly larger 
        font than the rest of the text, and the project name should be considered very important when determining the result.
        First provide the raw score as a number between 1 and 10 in square brackets, followed by an explanation of your reasoning.
        "###, self.job_title, self.job_role);
        println!("Prompt: {}", prompt);
        prompt
    }

}


fn get_init_script(screenshot_id: i64, productivity_score: i32, raw_llm_result: &str, png_image_path: &str, job_title: &str, job_role: &str) -> String {

    let raw_llm_result_base64: String = BASE64.encode(raw_llm_result);
    
    let png_image_path_base_64: String = BASE64.encode(png_image_path);

    let job_title_base_64: String = BASE64.encode(job_title);

    let job_role_base_64: String = BASE64.encode(job_role);

    format!(r#"    
        window.__SCREENTAP_SCREENSHOT__ = {{ id: '{}', productivity_score: {}, raw_llm_result_base64: '{}', png_image_path_base_64: '{}', job_title_base_64: '{}', job_role_base_64: '{}' }};
    "#, screenshot_id, 
        productivity_score, 
        raw_llm_result_base64, 
        png_image_path_base_64, 
        job_title_base_64, 
        job_role_base_64)
}

// Structs for payload
#[derive(Serialize)]
struct ImageUrl {
    url: String,
}

#[derive(Serialize)]
struct MessageContent {
    #[serde(rename = "type")]
    content_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    image_url: Option<ImageUrl>,
}

#[derive(Serialize)]
struct Message {
    role: String,
    content: Vec<MessageContent>,
}

#[derive(Serialize)]
struct Payload {
    model: String,
    messages: Vec<Message>,
    max_tokens: u32,
}
