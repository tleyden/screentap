
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
            duration_between_checks: Duration::from_secs(30),
            last_screentap_time: Instant::now(),
        }
    }

    pub fn handle_screentap_event(&self, png_data: Vec<u8>, ocr_text: String) {
        println!("Handling screentap event with len(ocr_text): {} and len(png_data): {}", ocr_text.len(), png_data.len());

        let now = Instant::now();
        let elapsed = now.duration_since(self.last_screentap_time);

        // Check if more than 5 minutes have passed
        if elapsed > self.duration_between_checks {
            println!("Time to check!");
        } else {
            println!("Not time to check. Only {} seconds have passed.", elapsed.as_secs());
        }


    }

}