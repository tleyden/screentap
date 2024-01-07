    // Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

extern crate screen_ocr_swift_rs;

use tauri::{Manager, SystemTray, SystemTrayEvent, SystemTrayMenu};

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

    let system_tray_menu = SystemTrayMenu::new();

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

    tauri::Builder::default()
    .system_tray(SystemTray::new().with_menu(system_tray_menu))
    .on_system_tray_event(|app, event| match event {
        SystemTrayEvent::LeftClick {
            position: _,
            size: _,
            ..
        } => {
            let window = app.get_window("main").unwrap();
            // toggle application window
            if window.is_visible().unwrap() {
                window.hide().unwrap();
            } else {
                window.show().unwrap();
                window.set_focus().unwrap();
            }
        },
        _ => {}
    })
    .invoke_handler(tauri::generate_handler![greet])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
    
}
