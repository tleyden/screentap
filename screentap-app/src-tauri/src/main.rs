    // Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

extern crate screen_ocr_swift_rs;

use tauri::{Manager, SystemTray, SystemTrayEvent, SystemTrayMenu, CustomMenuItem, SystemTrayMenuItem};

use std::collections::HashMap;
use std::thread;
use std::time::Duration;

mod db;
mod utils; 
mod screenshot;

const DATASET_ROOT: &str = "/Users/tleyden/Development/screentap/screentap-app/public/dataset";
const DATABASE_FILENAME: &str = "screentap.db";


#[tauri::command]
fn search_screenshots(term: &str) -> Vec<HashMap<String, String>> {

    println!("Searching for {}", term);

    let screenshot_records = db::get_all_screenshots(DATASET_ROOT, DATABASE_FILENAME);
    let screenshot_hashmaps = db::create_hashmap_vector(screenshot_records.unwrap().as_slice());

    // Loop over screenshot_records and convert to hashmaps


    // let mut results = Vec::new();

    // // Example data - replace with your actual OCR logic
    // results.push(HashMap::from([
    //     ("image_path".to_string(), "/dataset/2024_01_07_12_40_56.png".to_string()),
    //     ("ocr_text".to_string(), "Text from image 1".to_string()),
    // ]));
    // results.push(HashMap::from([
    //     ("image_path".to_string(), "/dataset/2024_01_07_12_40_56.png".to_string()),
    //     ("ocr_text".to_string(), "Text from image 2".to_string()),
    // ]));
    // results
    screenshot_hashmaps   
}

fn main() {

    let quit = CustomMenuItem::new("quit".to_string(), "Quit").accelerator("Cmd+Q");
    let show_hide_window = CustomMenuItem::new("show_hide_window".to_string(), "Show/Hide Screentap");

    let system_tray_menu = SystemTrayMenu::new()
        .add_item(show_hide_window)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(quit);

    // Create the database if it doesn't exist
    match db::create_db(DATASET_ROOT, DATABASE_FILENAME) {
        Ok(()) => println!("Ensured DB exists"),
        Err(e) => eprintln!("Failed to create db: {}", e),
    }

    // Spawn a thread to save screenshots in the background
    thread::spawn(|| {

        loop {
            println!("Saving screenshot in background thread ..");
            let _ = screenshot::save_screenshot(DATASET_ROOT, DATABASE_FILENAME);

            let sleep_time_secs = 300;
            println!("Sleeping for {} secs ..", sleep_time_secs);
            thread::sleep(Duration::from_secs(sleep_time_secs));
        }
    });

    tauri::Builder::default()
    .system_tray(SystemTray::new().with_menu(system_tray_menu))
    .on_system_tray_event(|app, event| match event {
        SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
            "quit" => {
                std::process::exit(0);
            },
            "show_hide_window" => {
                let window = app.get_window("main").unwrap();
                // toggle application window
                if window.is_visible().unwrap() {
                    window.hide().unwrap();
                } else {
                    window.show().unwrap();
                    window.set_focus().unwrap();
                }
            }
            _ => {}
        },
        _ => {}
    })
    .invoke_handler(tauri::generate_handler![search_screenshots])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
    
}
