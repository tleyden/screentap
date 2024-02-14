
use std::time::{Instant, Duration};

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

            // let raw_result = self.invoke_vision_model(prompt);

            // let productivity_score = self.process_vision_model_result(raw_result);

            // if productivity_score < 5 {
            //     println!("Productivity score is low: {}", productivity_score);
            // } else {
            //     println!("Productivity score is high: {}", productivity_score);
            // }

            self.last_screentap_time = now;

        } 

    }

    fn invoke_vision_model(&self, prompt: String) -> String {
        

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