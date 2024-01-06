// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

extern crate screen_ocr_swift_rs;
use screen_ocr_swift_rs::extract_text;
use screen_ocr_swift_rs::screen_capture;

use chrono::{DateTime, Local};
use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::Write;

#[tauri::command]
fn greet(name: &str) -> String {
    let now = Local::now();
    let timestamp_png_filename = generate_filename(now, "png");
    let dataset_root = Path::new("/Users/tleyden/Development/screentap/dataset");
    let target_png_file_path = dataset_root.join(timestamp_png_filename);
    screen_capture(target_png_file_path.to_str().unwrap());
    // let ocr_text = extract_text("/Users/tleyden/Development/screentap/screentap-app/screentap_test_screenshot.png");
    let ocr_text = extract_text(target_png_file_path.to_str().unwrap());
    let result = format!("Hello, {} - {}", name, ocr_text);
    let timestamp_ocr_text_filename = generate_filename(now, "txt");
    let target_ocr_text_file_path = dataset_root.join(timestamp_ocr_text_filename);

    match write_string_to_file(target_ocr_text_file_path, ocr_text.to_string().as_str()) {
        Ok(()) => println!("Content written to file successfully."),
        Err(e) => eprintln!("Failed to write to file: {}", e),
    }

    result
}

/**
 * Helper function to generate a filename based on the current time
 */
fn generate_filename(now: DateTime<Local>, extension: &str) -> PathBuf {

    let formatted_time = now.format("%Y_%m_%d_%H_%M_%S").to_string();
    let filename = format!("{}.{}", formatted_time, extension);
    PathBuf::from(filename)
}

/**
 * Helper function to write a string to a file
 */
fn write_string_to_file<P: AsRef<Path>>(file_path: P, content: &str) -> std::io::Result<()> {
    let mut file = File::create(file_path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
