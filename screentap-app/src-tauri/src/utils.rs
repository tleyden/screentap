use std::path::PathBuf;
use chrono::NaiveDateTime;


/**
 * Helper function to generate a filename based on the current time
 */
pub fn generate_filename(now: NaiveDateTime, extension: &str) -> PathBuf {

    let formatted_time = now.format("%Y_%m_%d_%H_%M_%S").to_string();
    let filename = format!("{}.{}", formatted_time, extension);
    PathBuf::from(filename)
}
