
use std::time::{Instant, Duration};
use serde::Serialize;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use reqwest::blocking::Response;
use tauri::Manager;
use serde_json::json;
use std::fmt;
use image::{GenericImageView, imageops::FilterType};
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::env;
use std::str::FromStr;

pub mod config;

const DEV_MODE: bool = true;

// Create an enum with three possible values: openai, llamafile, and ollama
#[allow(dead_code)]
#[derive(PartialEq)]
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

pub struct FocusGuard {
    pub job_title: String,
    pub job_role: String,
    pub openai_api_key: String,

    // The duration between focusguard checks (Vision Model invocations)
    pub duration_between_checks: Duration,

    // How long to delay before showing next distraction alert (eg, 30 mins)
    duration_between_alerts: Duration,

    // Track the last time a screentap event was handled
    last_screentap_time: Instant,

    // Track the last time a distraction alert was shown
    last_distraction_alert_time: Instant,

    // The backend to use for the LLaVA model
    llava_backend: LlavaBackendType,

    // The threshold to be considered in "flow state"
    productivity_score_threshold: i32,

    // The size of the largest image dimension (width or height) to send to the vision model.
    // For OpenAI, the max is 2000 pixels.  Using a smaller value will cause the model
    // to consume less tokens during inference. 
    image_dimension_longest_side: u32,

    // The path to the app data directory in order to find plugin assets like
    // the Llamafile binary
    app_data_dir: PathBuf,

}

impl FocusGuard {

    pub fn new_from_config(app_data_dir: PathBuf) -> Option<FocusGuard> {

        // Register plugin - create a new focusguard struct
        let openai_api_key = match env::var("OPENAI_API_KEY") {
            Ok(open_api_key_val) => open_api_key_val,
            Err(_) => "".to_string()
        };

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

                if llava_backend == LlavaBackendType::OpenAI && openai_api_key.is_empty() {
                    println!("OpenAI API key is required for OpenAI backend");
                    return None
                };

                FocusGuard::new(
                    config.job_title,
                    config.job_role,
                    openai_api_key,
                    config.duration_between_checks_secs,
                    config.duration_between_alerts_secs,
                    llava_backend,
                    config.productivity_score_threshold,
                    config.image_dimension_longest_side,
                    app_data_dir
                )
            },
            None => {
                println!("Unable to load FocusGuard config.  This plugin will not be enabled");
                return None
            }   
        };

        Some(focus_guard)

    }

    pub fn new(job_title: String, 
        job_role: String, 
        openai_api_key: String, 
        duration_between_checks_secs: u64,
        duration_between_alerts_secs: u64,
        llava_backend: LlavaBackendType,
        productivity_score_threshold: i32,
        image_dimension_longest_side: u32,
        app_data_dir: PathBuf
    ) -> FocusGuard {

        let duration_between_checks = Duration::from_secs(duration_between_checks_secs);
        let duration_between_alerts = Duration::from_secs(duration_between_alerts_secs);

        // Initialize tracking vars so that it begins with an initial check
        let last_screentap_time = Instant::now() - duration_between_checks - Duration::from_secs(1);
        let last_distraction_alert_time = Instant::now() - duration_between_alerts - Duration::from_secs(1);

        FocusGuard {
            job_title,
            job_role,
            openai_api_key,
            duration_between_checks: duration_between_checks,
            duration_between_alerts: duration_between_alerts,
            last_screentap_time: last_screentap_time,
            last_distraction_alert_time: last_distraction_alert_time,
            llava_backend: llava_backend,
            productivity_score_threshold: productivity_score_threshold,
            image_dimension_longest_side: image_dimension_longest_side,
            app_data_dir
        }

    }

    /**
     * The logic is as follows:
     * 
     * - First check if enough time has elapsed since the last distraction alert.  If not, return false.
     * - Then check if either of the following conditions are true:
     *   - The frontmost app has changed
     *   - Enough time has elapsed since the last screentap event was processed
     * 
     */
    pub fn should_invoke_vision_model(&self, now: Instant, frontmost_app_changed: bool) -> bool {

        // Check if enough time elapsed since last distraction alert.  We don't want to hound the user
        // with alerts if they already know they're in a distraction state
        let elapsed_alert = now.duration_since(self.last_distraction_alert_time);
        let enough_time_elapsed_alert = elapsed_alert > self.duration_between_alerts;
        if !enough_time_elapsed_alert {
            println!("Not enough time elapsed since last distraction alert.  elapsed_alert: {:?}", elapsed_alert);
            return false
        }

        // If the frontmost app changed, then we should invoke the vision model immediately, since it is 
        // much more likely that the user has entered a distraction zone
        if frontmost_app_changed {
            println!("Frontmost app changed, so invoking vision model");
            return true
        }
                
        // If we got this far, check if enough time elapsed between checks to justify invoking the vision model  
        let elapsed = now.duration_since(self.last_screentap_time);
        let enough_time_elapsed = elapsed > self.duration_between_checks;
        if enough_time_elapsed {
            println!("Enough time elapsed since last screentap event and last distraction alert.  elapsed_alert: {:?} elapsed: {:?} ", elapsed_alert, elapsed);
        } else {
            println!("Not enough time elapsed since last screentap event and last distraction alert.  elapsed_alert: {:?} elapsed: {:?} ", elapsed_alert, elapsed);
        }

        enough_time_elapsed

    }

    pub fn handle_screentap_event(&mut self, app: &tauri::AppHandle, png_data: Vec<u8>, png_image_path: &Path, screenshot_id: i64, ocr_text: String, frontmost_app: &str, frontmost_app_changed: bool) {

        println!("FocusGuard handling screentap event # {} with len(ocr_text): {} and len(png_data): {} frontmost app: {}", screenshot_id, ocr_text.len(), png_data.len(), frontmost_app);

        // Get the current time
        let now = Instant::now();

        if !self.should_invoke_vision_model(now, frontmost_app_changed) {
            return
        }

        self.last_screentap_time = now;

        let prompt = self.create_prompt();

        let productivity_score = match DEV_MODE {
            true => {
                println!("FocusGuard returning hardcoded productivity score");
                2
            },  
            false => {
                // Invoke the actual vision model
                println!("FocusGuard analyzing image with {}.  Resizing image ..", self.llava_backend);

                let now = Instant::now();

                // Resize the image before sending to the vision model
                // TODO: just capture image in target dimensions in the first place, this will save a lot of resources.
                // TODO: or if that's not possible, move the resizing to native swift libraries to take adcantage of apple silicon
                let resize_img_result = FocusGuard::resize_image(
                    png_data, 
                    self.image_dimension_longest_side
                );

                // Get the resized png data
                let resized_png_data = match resize_img_result {
                    Ok(resized_img) => resized_img,
                    Err(e) => {
                        println!("Error resizing image: {}", e);
                        return
                    }
                };

                let time_to_resize = now.elapsed();
                println!("Resized image length in bytes: {}: time_to_resize: {:?}", resized_png_data.len(), time_to_resize);

                let now2 = Instant::now();

                let raw_result = match self.llava_backend {
                    LlavaBackendType::OpenAI => self.invoke_openai_vision_model(&prompt, &resized_png_data),
                    LlavaBackendType::Ollama => self.invoke_ollama_vision_model(&prompt, &resized_png_data),
                    LlavaBackendType::LlamaFile => self.invoke_openai_vision_model(&prompt, &resized_png_data),
                    LlavaBackendType::LlamaFileSubprocess => self.invoke_subprocess_vision_model(&prompt, &png_image_path),
                };

                let time_to_infer = now2.elapsed();
                println!("time_to_infer: {:?}", time_to_infer);

                match self.process_vision_model_result(&raw_result) { 
                    Some(raw_result_i32) => {
                        raw_result_i32
                    },
                    None => {
                        println!("FocusGuard could not parsing raw result [{}] into number", raw_result);
                        return
                    }
                }

            }
        };

        if DEV_MODE || productivity_score < self.productivity_score_threshold {
            println!("Productivity score is low: {} for png_image_path: {}", productivity_score, png_image_path.display());

            self.show_productivity_alert(app, productivity_score, png_image_path, screenshot_id);

            self.last_distraction_alert_time = Instant::now();


        } else {
            println!("Woohoo!  Looks like you're working.  Score is: {} for png_image_path: {}", productivity_score, png_image_path.display());
        }


    }


    fn resize_image(png_data: Vec<u8>, max_dimension: u32) -> Result<Vec<u8>, image::ImageError> {

        // Load the image from a byte slice (&[u8])
        let img = image::load_from_memory(&png_data)?;
    
        // Calculate the new dimensions
        let (width, height) = img.dimensions();
        let aspect_ratio = width as f32 / height as f32;
        let (new_width, new_height) = if width > height {
            let new_width = max_dimension;
            let new_height = (max_dimension as f32 / aspect_ratio).round() as u32;
            (new_width, new_height)
        } else {
            let new_height = max_dimension;
            let new_width = (max_dimension as f32 * aspect_ratio).round() as u32;
            (new_width, new_height)
        };
    
        // Resize the image
        let resized = img.resize_exact(
            new_width, 
            new_height, 
            FilterType::Lanczos3
        );
    
        let mut bytes = Cursor::new(Vec::new());
        resized.write_to(&mut bytes, image::ImageOutputFormat::Png)?;
        Ok(bytes.into_inner())

    }



    fn show_productivity_alert(&self, app: &tauri::AppHandle, productivity_score: i32, png_image_path: &Path, screenshot_id: i64) {

        // TODO: pass the score to the UI somehow
        println!("Showing productivity alert for score: {}", productivity_score);

        let window = app.get_window("focusguard");
        match window {
            Some(w) => {

                println!("Window exists, showing existing productivity alert window");

                // Window exists, so just bring it to the foreground
                w.show().unwrap();
                w.set_focus().unwrap();
                
                let event_name = "my-custom-event"; // The event name to emit
                let payload = serde_json::json!({
                    "message": "Hello from Rust! - send event to current window"
                }); // Payload to send with the event, serialized as JSON

                // Emitting the event to the JavaScript running in the window
                if let Err(e) = w.emit(event_name, Some(payload)) {
                    eprintln!("Error emitting event: {}", e);
                }
            },
            None => {

                println!("Window does not exist, creating and showing new productivity alert window");

                // const INIT_SCRIPT: &str = r#"
                //   console.log("hello world from js init script", window.location.origin);
              
                //   window.__MY_CUSTOM_PROPERTY__ = { foo: 'bar' };

                //   const button = document.createElement('button');
                //   button.textContent = 'Click me too!!';
                //   document.body.appendChild(button);

                // "#;

                let init_script = get_init_script(png_image_path.to_str().unwrap(), screenshot_id);
                println!("init_script: {}", init_script);

                // Create and show new window
                let w = tauri::WindowBuilder::new(
                    app,
                    "focusguard",
                    tauri::WindowUrl::App("index_focusguard.html".into())
                ).initialization_script(&init_script).maximized(true).title("Focusguard").build().expect("failed to build window");

                let event_name = "my-custom-event"; // The event name to emit
                let payload = serde_json::json!({
                    "message": "Hello from Rust - create and show new window!"
                }); // Payload to send with the event, serialized as JSON

                // Emitting the event to the JavaScript running in the window
                if let Err(e) = w.emit(event_name, Some(payload)) {
                    eprintln!("Error emitting event: {}", e);
                }

            }
        }   

    }
    
    fn process_vision_model_result(&self, raw_llm_response: &str) -> Option<i32> {
        // Try to convert the raw result into a number
        match raw_llm_response.parse::<i32>() {
            Ok(raw_result_i32) => Some(raw_result_i32),
            Err(e) => {
                println!("Error parsing raw LLM response {} into number: {}", raw_llm_response, e);
                None
            }
        }
    }

    fn convert_png_data_to_base_64(&self, png_data: &Vec<u8>) -> String {
        let base64_image = base64::encode(png_data);
        base64_image
    }

    /**
     * To run it with ollama, you need to have it running on localhost:11434.  
     * 
     * $ ollama run llava
     * $ ctl-c
     * $ ollama serve
     */
    fn invoke_ollama_vision_model(&self, prompt: &str, png_data: &Vec<u8>) -> String {

        // Getting the base64 string
        let base64_image = self.convert_png_data_to_base_64(png_data);

        let client = reqwest::blocking::Client::new();

        let payload = json!({
            "model": "llava",
            "prompt": prompt.to_string(),
            "stream": false,
            "images": [base64_image]
        });

        let target_url = "http://localhost:11434/api/generate";

        let response_result = client.post(target_url)
            .json(&payload)
            .send();

        let response = match response_result {
            Ok(response) => response,
            Err(e) => {
                println!("Error invoking vision model: {}", e);
                return "".to_string();
            }
        };

        let response_json = match response.json::<serde_json::Value>() {
            Ok(response_json) => response_json,
            Err(e) => {
                println!("Error parsing response JSON: {}", e);
                return "".to_string();
            }
        };

        response_json["response"].to_string()

    }

    fn invoke_subprocess_vision_model(&self, prompt: &str, png_image_path: &Path) -> String {

        let full_prompt = format!("### User: {}\n ### Assistant:", prompt);

        // before: Command::new("/Users/tleyden/Development/screentap/screentap-app/src-tauri/llava-v1.5-7b-q4.llamafile")
        let llamafile_path = self.app_data_dir.join("plugins").join("focusguard").join("llava-v1.5-7b-q4.llamafile");
        if !llamafile_path.exists() {
            println!("Cannot find Llamafile at {}, skipping inference and returning empty string", llamafile_path.display());
            return "".to_string();
        } 

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
                if choices.len() == 0 {
                    println!("No choices in response");
                    return "".to_string();
                }
                &choices[0]
            },
            None => {
                println!("No choices in response");
                return "".to_string();
            }
        };

        let message_content = &first_choice["message"]["content"].as_str();

        message_content.unwrap_or("").to_string()

    }

    fn create_prompt(&self) -> String {
        let prompt = format!(r###"On a scale of 1 to 10, how much does this screenshot indicate 
        a worker with job title of "{}" and job role of "{}" is currently engaged in work activities?  
        When analyzing the screenshots, please note that:
        * In many apps such as VS Code and Slack, the project name is often displayed in the top left corner in a slightly larger 
        font than the rest of the text, and the project name should be considered very important when determining the result.
        Do not provide any explanation, just the score itself which is a number between 1 and 10."###, self.job_title, self.job_role);
        println!("Prompt: {}", prompt);
        prompt
    }

}


fn get_init_script(console_log: &str, screenshot_id: i64) -> String {
    format!(r#"
        console.log("hello world from js init script");
        console.log("console_log", "{}");
        console.log("screenshot_id", "{}");
    
        window.__MY_CUSTOM_PROPERTY__ = {{ foo: 'bar' }};

        const button = document.createElement('button');
        button.textContent = 'click me too';
        document.body.appendChild(button);
    "#, console_log, screenshot_id)
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