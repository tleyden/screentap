// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

extern crate screen_ocr_swift_rs;

use std::thread;
use std::time::Duration;

mod db;
mod utils; 
mod screenshot;

const DATASET_ROOT: &str = "/Users/tleyden/Development/screentap/dataset";
const DATABASE_FILENAME: &str = "screentap.db";

#[tauri::command]
fn greet() -> String {
    format!("No screenshot saved, running in background thread ..")
}

fn main() {

    // Spawn a thread to save screenshots in the background
    thread::spawn(|| {

        // Create the database if it doesn't exist
        match db::create_db(DATASET_ROOT, DATABASE_FILENAME) {
            Ok(()) => println!("Ensured DB exists"),
            Err(e) => eprintln!("Failed to create db: {}", e),
        }

        loop {
            println!("Saving screenshot in background thread ..");
            let _ = screenshot::save_screenshot(DATASET_ROOT, DATABASE_FILENAME);
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
