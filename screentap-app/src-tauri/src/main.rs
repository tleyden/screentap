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


const DATABASE_FILENAME: &str = "screentap.db";




#[tauri::command]
fn search_screenshots(app_handle: tauri::AppHandle, term: &str) -> Vec<HashMap<String, String>> {

    let app_data_dir = app_handle.path_resolver().app_data_dir().unwrap().to_str().unwrap().to_string();

    // Cap the max results until we implement techniques to reduce memory footprint
    let max_results = 25;

    let screenshot_records_result = if term.is_empty() {
        db::get_all_screenshots(app_data_dir.as_str(), DATABASE_FILENAME, max_results)
    } else {
        db::search_screenshots_ocr(term, app_data_dir.as_str(), DATABASE_FILENAME, max_results)
    };

    match screenshot_records_result {
        Ok(screenshot_records) => {
            db::create_hashmap_vector(screenshot_records.as_slice())
        },
        Err(e) => {
            println!("Error searching screenshots: {}.  Returning empty result", e);
            vec![]
        },
    }
}

fn setup_handler(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error + 'static>> {

    let app_handle = app.handle();

    println!("resource_dir: {}", app_handle.path_resolver().resource_dir().unwrap_or(std::path::PathBuf::new()).to_string_lossy());
    println!("app_config_dir: {}", app_handle.path_resolver().app_config_dir().unwrap_or(std::path::PathBuf::new()).to_string_lossy());
    println!("app_data_dir: {}", app_handle.path_resolver().app_data_dir().unwrap_or(std::path::PathBuf::new()).to_string_lossy());
    println!("app_local_data_dir: {}", app_handle.path_resolver().app_local_data_dir().unwrap_or(std::path::PathBuf::new()).to_string_lossy());
    println!("app_cache_dir: {}", app_handle.path_resolver().app_cache_dir().unwrap_or(std::path::PathBuf::new()).to_string_lossy());
    println!("app_log_dir: {}", app_handle.path_resolver().app_log_dir().unwrap_or(std::path::PathBuf::new()).to_string_lossy());
    println!("data_dir: {}", tauri::api::path::data_dir().unwrap_or(std::path::PathBuf::new()).to_string_lossy());
    println!("local_data_dir: {}", tauri::api::path::local_data_dir().unwrap_or(std::path::PathBuf::new()).to_string_lossy());
    println!("cache_dir: {}", tauri::api::path::cache_dir().unwrap_or(std::path::PathBuf::new()).to_string_lossy());
    println!("config_dir: {}", tauri::api::path::config_dir().unwrap_or(std::path::PathBuf::new()).to_string_lossy());
    println!("executable_dir: {}", tauri::api::path::executable_dir().unwrap_or(std::path::PathBuf::new()).to_string_lossy());
    println!("public_dir: {}", tauri::api::path::public_dir().unwrap_or(std::path::PathBuf::new()).to_string_lossy());
    println!("runtime_dir: {}", tauri::api::path::runtime_dir().unwrap_or(std::path::PathBuf::new()).to_string_lossy());
    println!("template_dir: {}", tauri::api::path::template_dir().unwrap_or(std::path::PathBuf::new()).to_string_lossy());
    println!("font_dir: {}", tauri::api::path::font_dir().unwrap_or(std::path::PathBuf::new()).to_string_lossy());
    println!("home_dir: {}", tauri::api::path::home_dir().unwrap_or(std::path::PathBuf::new()).to_string_lossy());
    println!("audio_dir: {}", tauri::api::path::audio_dir().unwrap_or(std::path::PathBuf::new()).to_string_lossy());
    println!("desktop_dir: {}", tauri::api::path::desktop_dir().unwrap_or(std::path::PathBuf::new()).to_string_lossy());
    println!("document_dir: {}", tauri::api::path::document_dir().unwrap_or(std::path::PathBuf::new()).to_string_lossy());
    println!("download_dir: {}", tauri::api::path::download_dir().unwrap_or(std::path::PathBuf::new()).to_string_lossy());
    println!("picture_dir: {}", tauri::api::path::picture_dir().unwrap_or(std::path::PathBuf::new()).to_string_lossy());

    let app_data_dir = app_handle.path_resolver().app_data_dir().unwrap().to_str().unwrap().to_string();
    // If app_data_dir doesn't exist, create it
    if !std::path::Path::new(&app_data_dir).exists() {
        println!("Creating app_data_dir: {}", app_data_dir);
        std::fs::create_dir_all(&app_data_dir)?;
    }

    // Create the database if it doesn't exist
    match db::create_db(app_data_dir.as_str(), DATABASE_FILENAME) {
        Ok(()) => println!("Ensured DB exists"),
        Err(e) => eprintln!("Failed to create db: {}", e),
    }

    // Spawn a thread to save screenshots in the background
    // The move keyword is necessary to move app_data_dir into the thread
    thread::spawn(move || {

        loop {
            println!("Saving screenshot in background thread ..");
            let _ = screenshot::save_screenshot(app_data_dir.as_str(), DATABASE_FILENAME);

            let sleep_time_secs = 120;
            println!("Sleeping for {} secs ..", sleep_time_secs);
            thread::sleep(Duration::from_secs(sleep_time_secs));
        }
    });

    Ok(())

}

fn browse_screenshots(app: &tauri::AppHandle) -> tauri::Window {
    let new_window = tauri::WindowBuilder::new(
        app,
        "browse",
        tauri::WindowUrl::App("index.html".into())
    ).build().expect("failed to build window");

    new_window
}


fn main() {

    let quit = CustomMenuItem::new("quit".to_string(), "Quit").accelerator("Cmd+Q");
    let show_hide_window = CustomMenuItem::new("show_hide_window".to_string(), "Show/Hide Screentap");
    let browse_screenshots_menu_item = CustomMenuItem::new("browse_screenshots".to_string(), "Browse");

    let system_tray_menu = SystemTrayMenu::new()
        .add_item(show_hide_window)
        .add_item(browse_screenshots_menu_item)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(quit);

    tauri::Builder::default()
    .setup(setup_handler)
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
            },
            "browse_screenshots" => {
                let _ = browse_screenshots(&app);
            },
            _ => {}
        },
        _ => {}
    })
    .invoke_handler(tauri::generate_handler![search_screenshots])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
    
}
