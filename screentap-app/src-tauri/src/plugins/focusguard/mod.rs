
use std::time::{Instant, Duration};
use serde::Serialize;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};


pub struct FocusGuard {
    pub job_title: String,
    pub job_role: String,
    pub openai_api_key: String,

    // The duration between focusguard checks
    pub duration_between_checks: Duration,

    // Internal tracking variable to track the last time a screentap event was handled
    last_screentap_time: Instant,

}

impl FocusGuard {

    pub fn new(job_title: String, job_role: String, openai_api_key: String) -> FocusGuard {
        FocusGuard {
            job_title,
            job_role,
            openai_api_key,
            duration_between_checks: Duration::from_secs(10),
            last_screentap_time: Instant::now(),
        }
    }

    pub fn handle_screentap_event(&mut self, png_data: Vec<u8>, ocr_text: String) {
        println!("Handling screentap event with len(ocr_text): {} and len(png_data): {}", ocr_text.len(), png_data.len());

        let now = Instant::now();
        let elapsed = now.duration_since(self.last_screentap_time);

        if elapsed > self.duration_between_checks {
            println!("Time to check!");

            let prompt = self.create_prompt();

            let raw_result = self.invoke_vision_model(&prompt, &png_data);
            println!("Raw result: {}", raw_result);

            // let productivity_score = self.process_vision_model_result(raw_result);

            // if productivity_score < 5 {
            //     println!("Productivity score is low: {}", productivity_score);
            // } else {
            //     println!("Productivity score is high: {}", productivity_score);
            // }

            self.last_screentap_time = now;

        } 

    }

    fn convert_png_data_to_base_64(&self, png_data: &Vec<u8>) -> String {
        let base64_image = base64::encode(png_data);
        base64_image
    }

    fn invoke_vision_model(&self, prompt: &str, png_data: &Vec<u8>) -> String {

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

        println!("Headers: {:?}", headers);

        let payload = Payload {
            model: "gpt-4-vision-preview".to_string(),
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

        let response_result = client.post("https://api.openai.com/v1/chat/completions")
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

        let response_json = match response.json::<serde_json::Value>() {
            Ok(response_json) => response_json,
            Err(e) => {
                println!("Error parsing response JSON: {}", e);
                return "".to_string();
            }
        };
    
        println!("response_json: {:?}", response_json);

        response_json["choices"][0]["message"]["content"].to_string()
        
    }

    fn create_prompt(&self) -> String {
        let prompt = format!(r###"Imagine you are a boss or a coworker looking at this
        screen of an employee or colleague.  On a scale of 1 to 10, how much does this screenshot indicate 
        a worker with job title of "{}" and job role of "{}" is currently engaged in work activities?  Do not 
        provide any explanation, just the score itself."###, self.job_title, self.job_role);
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