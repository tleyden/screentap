use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use chrono::{DateTime, Local};


/**
 * Helper function to generate a filename based on the current time
 */
pub fn generate_filename(now: DateTime<Local>, extension: &str) -> PathBuf {

    let formatted_time = now.format("%Y_%m_%d_%H_%M_%S").to_string();
    let filename = format!("{}.{}", formatted_time, extension);
    PathBuf::from(filename)
}

/**
 * Helper function to write a string to a file
 */
pub fn write_string_to_file<P: AsRef<Path>>(file_path: P, content: &str) -> std::io::Result<()> {
    let mut file = File::create(file_path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}