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


/**
 * Get the name of the frontmost app via applescript
 */
pub fn get_frontmost_app_via_applescript() -> String {
    let script = r#"
        tell application "System Events" to tell (first process whose frontmost is true) to return name
    "#;
    let output = std::process::Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()
        .expect("Failed to execute osascript");
    let output_str = String::from_utf8_lossy(&output.stdout);
    output_str.trim().to_string()
}