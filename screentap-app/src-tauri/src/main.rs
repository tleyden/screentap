// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

extern crate screen_ocr_swift_rs;
use screen_ocr_swift_rs::extract_text;
use screen_ocr_swift_rs::screen_capture;

use chrono::Local;
use std::path::{Path, PathBuf};

#[tauri::command]
fn greet(name: &str) -> String {
    let timestamp_filename = generate_filename("png");
    let target_file_path = Path::new("/Users/tleyden/Development/screentap/dataset").join(timestamp_filename);
    screen_capture(target_file_path.to_str().unwrap());
    // let ocr_text = extract_text("/Users/tleyden/Development/screentap/screentap-app/screentap_test_screenshot.png");
    let ocr_text = extract_text(target_file_path.to_str().unwrap());
    let result = format!("Hello, {} - {}", name, ocr_text);
    result
}

/**
 * Helper function to generate a filename based on the current time
 */
fn generate_filename(extension: &str) -> PathBuf {
    let now = Local::now();
    let formatted_time = now.format("%Y_%m_%d_%H_%M_%S").to_string();
    let filename = format!("{}.{}", formatted_time, extension);
    PathBuf::from(filename)
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
