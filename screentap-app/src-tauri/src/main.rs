// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

extern crate screen_ocr_swift_rs;
use screen_ocr_swift_rs::extract_text;
use screen_ocr_swift_rs::screen_capture;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    screen_capture();
    let ocr_text = extract_text("/Users/tleyden/Development/screentap/screentap-app/screentap_test_screenshot.png");
    let result = format!("Hello, {} - {}", name, ocr_text);
    result
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
