// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

extern crate screen_ocr_swift_rs;
use screen_ocr_swift_rs::extract_text;
use screen_ocr_swift_rs::screen_capture;

use chrono::{DateTime, Local, Utc};
use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::Write;
use rusqlite::{params, Connection, Result};
use std::thread;
use std::time::Duration;

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
    let timestamp_png_filename = generate_filename(now, "png");
    let dataset_root_path = Path::new(DATASET_ROOT);
    let target_png_file_path = dataset_root_path.join(timestamp_png_filename);
    screen_capture(target_png_file_path.to_str().unwrap());
    let ocr_text = extract_text(target_png_file_path.to_str().unwrap());
    let timestamp_ocr_text_filename = generate_filename(now, "txt");
    let target_ocr_text_file_path = dataset_root_path.join(timestamp_ocr_text_filename);

    match write_string_to_file(target_ocr_text_file_path, ocr_text.to_string().as_str()) {
        Ok(()) => println!("Content written to file successfully."),
        Err(e) => eprintln!("Failed to write to file: {}", e),
    }

    // Save screenshot meta to the DB
    let save_result = save_screenshot_meta(
        target_png_file_path.to_str().unwrap(), 
        ocr_text.to_string().as_str()
    );

    let current_time_formatted = now.format("%Y-%m-%d %H:%M:%S").to_string();
    let result = match save_result {
        Ok(()) => format!("Screenshot saved to DB successfully at {}", current_time_formatted),
        Err(e) => format!("Error occurred: {} at {}", e, current_time_formatted),
    };

    result  // TODO: return the save_result instead of this string
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

/**
 * Helper function to create the DB if it doesn't exist
 */
fn create_db(db_filename: &str) -> Result<()> {

    let conn = Connection::open(db_filename)?;

    // Create a table with the desired columns
    conn.execute(
        "CREATE TABLE IF NOT EXISTS documents (
                id INTEGER PRIMARY KEY,
                timestamp TIMESTAMP NOT NULL,
                ocr_text TEXT NOT NULL,
                file_path TEXT NOT NULL
            )",
        [],
    )?;

    // Enable full-text search on the OCR text column
    conn.execute(
        "CREATE VIRTUAL TABLE IF NOT EXISTS ocr_text_index 
            USING fts5(ocr_text);",
        [],
    )?;

    Ok(())

}


/**
 * Helper function to save screenshot meta to the DB
 */
fn save_screenshot_meta(screenshot_file_path: &str, ocr_text: &str) -> Result<()> {

    let dataset_root_path = Path::new(DATASET_ROOT);
    let db_filename = dataset_root_path.join(DATABASE_FILENAME);
    let conn = Connection::open(db_filename.to_str().unwrap())?;

    let now = Utc::now().naive_utc();

    conn.execute(
        "INSERT INTO documents (timestamp, ocr_text, file_path) VALUES (?1, ?2, ?3)",
        params![now.timestamp(), ocr_text, screenshot_file_path],
    )?;

    // Insert the OCR text into the full-text search index
    conn.execute(
        "INSERT INTO ocr_text_index (ocr_text) VALUES (?1)",
        [ocr_text],
    )?;

    Ok(())

}

fn main() {

    // Create the database if it doesn't exist
    let dataset_root_path = Path::new(DATASET_ROOT);
    let db_filename = dataset_root_path.join(DATABASE_FILENAME);
    println!("Creating db_filename: {} if it doesn't exist", db_filename.to_str().unwrap());
    let db_create_result = create_db(db_filename.to_str().unwrap());
    match db_create_result {
        Ok(()) => println!("Created db"),
        Err(e) => eprintln!("Failed to create db: {}", e),
    }

    // Spawn a thread to save screenshots in the background
    thread::spawn(|| {
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
