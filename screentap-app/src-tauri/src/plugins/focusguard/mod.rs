
use std::time::{Instant, Duration};
use serde::Serialize;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use reqwest::blocking::Response;
use tauri::Manager;
use serde_json::json;
use std::fmt;
use image::{GenericImageView, imageops::FilterType};
use std::io::Cursor;
use std::path::Path;
use std::process::Command;
use std::env;
use std::str::FromStr;

pub mod config;

const DEV_MODE: bool = false;

// Create an enum with three possible values: openai, llamafile, and ollama
#[allow(dead_code)]
enum LlavaBackendType {

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

}

impl FocusGuard {

    pub fn new_from_config(app_data_dir: &Path) -> Option<FocusGuard> {

        // Register plugin - create a new focusguard struct
        let openai_api_key = match env::var("OPENAI_API_KEY") {
            Ok(open_api_key_val) => open_api_key_val,
            Err(_) => "".to_string()
        };

        let focus_guard_config = config::FocusGuardConfig::new(app_data_dir);
        let focus_guard = match focus_guard_config {
            Some(config) => {
                FocusGuard::new(
                    config.job_title,
                    config.job_role,
                    openai_api_key,
                    config.duration_between_checks_secs,
                    config.duration_between_alerts_secs,
                    config.llava_backend,
                    config.productivity_score_threshold,
                    config.image_dimension_longest_side
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
        llava_backend_str: String,
        productivity_score_threshold: i32,
        image_dimension_longest_side: u32,
    ) -> FocusGuard {

        let duration_between_checks = Duration::from_secs(duration_between_checks_secs);
        let duration_between_alerts = Duration::from_secs(duration_between_alerts_secs);

        // Initialize tracking vars so that it begins with an initial check
        let last_screentap_time = Instant::now() - duration_between_checks - Duration::from_secs(1);
        let last_distraction_alert_time = Instant::now() - duration_between_alerts - Duration::from_secs(1);

        let llava_backend = LlavaBackendType::from_str(&llava_backend_str).expect("Failed to parse vision model backend type");
        
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
            image_dimension_longest_side: image_dimension_longest_side
        }

    }

    pub fn should_invoke_vision_model(&self, now: Instant) -> bool {
        
        // Check if enough time elapsed between checks
        let elapsed = now.duration_since(self.last_screentap_time);
        let enough_time_elapsed = elapsed > self.duration_between_checks;

        // Check if enough time elapsed since last distraction alert
        let elapsed_alert = now.duration_since(self.last_distraction_alert_time);
        let enough_time_elapsed_alert = elapsed_alert > self.duration_between_alerts;

        enough_time_elapsed && enough_time_elapsed_alert

    }

    pub fn handle_screentap_event(&mut self, app: &tauri::AppHandle, png_data: Vec<u8>, png_image_path: &Path, ocr_text: String) {

        println!("FocusGuard handling screentap event with len(ocr_text): {} and len(png_data): {}", ocr_text.len(), png_data.len());

        // Get the current time
        let now = Instant::now();

        if !self.should_invoke_vision_model(now) {
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
                println!("FocusGuard analyzing image with {}", self.llava_backend);

                // Resize the image before sending to the vision model
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

                println!("Resized image length in bytes: {}", resized_png_data.len());

                let raw_result = match self.llava_backend {
                    LlavaBackendType::OpenAI => self.invoke_openai_vision_model(&prompt, &resized_png_data),
                    LlavaBackendType::Ollama => self.invoke_ollama_vision_model(&prompt, &resized_png_data),
                    LlavaBackendType::LlamaFile => self.invoke_openai_vision_model(&prompt, &resized_png_data),
                    LlavaBackendType::LlamaFileSubprocess => self.invoke_subprocess_vision_model(&prompt, &png_image_path),
                };

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

        if productivity_score < self.productivity_score_threshold {
            println!("Productivity score is low: {}", productivity_score);

            self.show_productivity_alert(app, productivity_score);

            self.last_distraction_alert_time = Instant::now();


        } else {
            println!("Woohoo!  Looks like you're working.  Score is: {}", productivity_score);
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

    fn show_productivity_alert(&self, app: &tauri::AppHandle, productivity_score: i32) {


        // TODO: pass the score to the UI somehow
        println!("Showing productivity alert for score: {}", productivity_score);

        let window = app.get_window("focusguard");
        match window {
            Some(w) => {
                // Window exists, so just bring it to the foreground
                w.show().unwrap();
                w.set_focus().unwrap();
            },
            None => {
                // Create and show new window
                let _ = tauri::WindowBuilder::new(
                    app,
                    "focusguard",
                    tauri::WindowUrl::App("index_focusguard.html".into())
                ).maximized(true).title("Focusguard").build().expect("failed to build window");
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

        // sh ./llava-v1.5-7b-q4.llamafile -ngl 9999 --image ~/Desktop/2024_02_15_10_24_53.png -e -p '### User: On a scale of 1 to 10, how much does this screenshot indicate..'
        let output = Command::new("/Users/tleyden/Development/screentap/screentap-app/src-tauri/llava-v1.5-7b-q4.llamafile")
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
        a worker with job title of "{}" and job role of "{}" is currently engaged in work activities?  Do not 
        provide any explanation, just the score itself which is a number between 1 and 10."###, self.job_title, self.job_role);
        println!("Prompt: {}", prompt);
        prompt
    }

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