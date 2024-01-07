// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

extern crate screen_ocr_swift_rs;

use chrono::Local;
use std::path::Path;
use std::thread;
use std::time::Duration;

mod db;
mod utils; 

const DATASET_ROOT: &str = "/Users/tleyden/Development/screentap/dataset";
const DATABASE_FILENAME: &str = "screentap.db";

#[tauri::command]
fn greet() -> String {
    format!("No screenshot saved, running in background thread ..")
}

/**
 * Helper function to save a screenshot and OCR text to the dataset directory and DB
 */
fn save_screenshot() -> String {

    let now = Local::now();
    let timestamp_png_filename = utils::generate_filename(now, "png");
    let dataset_root_path = Path::new(DATASET_ROOT);
    let target_png_file_path = dataset_root_path.join(timestamp_png_filename);

    screen_ocr_swift_rs::screen_capture(target_png_file_path.to_str().unwrap());
    let ocr_text = screen_ocr_swift_rs::extract_text(target_png_file_path.to_str().unwrap());
    
    let timestamp_ocr_text_filename = utils::generate_filename(now, "txt");
    let target_ocr_text_file_path = dataset_root_path.join(timestamp_ocr_text_filename);

    match utils::write_string_to_file(target_ocr_text_file_path, ocr_text.to_string().as_str()) {
        Ok(()) => println!("Content written to file successfully."),
        Err(e) => eprintln!("Failed to write to file: {}", e),
    }

    // Save screenshot meta to the DB
    let save_result = db::save_screenshot_meta(
        target_png_file_path.to_str().unwrap(), 
        ocr_text.to_string().as_str(),
        DATASET_ROOT,
        DATABASE_FILENAME
    );

    let current_time_formatted = now.format("%Y-%m-%d %H:%M:%S").to_string();
    let result = match save_result {
        Ok(()) => format!("Screenshot saved to DB successfully at {}", current_time_formatted),
        Err(e) => format!("Error occurred: {} at {}", e, current_time_formatted),
    };

    result  // TODO: return the save_result instead of this string
}


fn main() {

    // Spawn a thread to save screenshots in the background
    thread::spawn(|| {

        // Create the database if it doesn't exist
        match db::create_db(DATASET_ROOT, DATABASE_FILENAME) {
            Ok(()) => println!("Created db"),
            Err(e) => eprintln!("Failed to create db: {}", e),
        }

        loop {
            println!("Saving screenshot in background thread ..");
            let _ = save_screenshot();
            let sleep_time_secs = 300;
            println!("Sleeping for {} secs ..", sleep_time_secs);
            thread::sleep(Duration::from_secs(sleep_time_secs));
        }
    });

    // Run the Tauri event loop
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
    
}
