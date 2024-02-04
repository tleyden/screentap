// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

extern crate screen_ocr_swift_rs;

use tauri::{Manager, SystemTray, SystemTrayEvent, SystemTrayMenu, CustomMenuItem, SystemTrayMenuItem};

use core::time;
use std::collections::HashMap;
use std::thread;
use std::time::Duration;
use std::path::Path;
use std::env;
use std::path::PathBuf;
use chrono::Local;



mod db;
mod utils; 
mod screenshot;
mod compaction;

static DATABASE_FILENAME: &str = "screentap.db";

#[tauri::command]
fn search_screenshots(app_handle: tauri::AppHandle, term: &str) -> Vec<HashMap<String, String>> {

    // let app_data_dir = app_handle.path_resolver().app_data_dir().expect("Failed to get app_data_dir");
    let app_data_dir = get_effective_app_dir(app_handle);

    let db_filename_path = Path::new(DATABASE_FILENAME);

    // Cap the max results until we implement techniques to reduce memory footprint
    let max_results: i32 = 25;

    let screenshot_records_result = if term.is_empty() {
        db::get_all_screenshots(app_data_dir.as_path(), db_filename_path, max_results)
    } else {
        db::search_screenshots_ocr(term, app_data_dir.as_path(), db_filename_path, max_results)
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

#[tauri::command]
fn browse_screenshots(app_handle: tauri::AppHandle, cur_id: i32, direction: &str) -> Vec<HashMap<String, String>> {

    println!("browse_screenshots: cur_id: {}, direction: {}", cur_id, direction);

    let app_data_dir = get_effective_app_dir(app_handle);

    let db_filename_path = Path::new(DATABASE_FILENAME);

    let screenshot_records_result = match cur_id {
        0 => {
            // If the user passed 0 as the cur_id, get the most recent screenshot in the DB
            db::get_all_screenshots(
                app_data_dir.as_path(), 
                db_filename_path, 
                1
            )
        },
        _ => {
            // Otherwise, get the next screenshot by id, depending on direction
            let target_id = match direction {
                "forward" => cur_id + 1,
                "backward" => cur_id - 1,
                _ => cur_id,
            };
            db::get_screenshot_by_id(
                app_data_dir.as_path(), 
                db_filename_path, 
                target_id
            )
        }
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


fn get_effective_app_dir(app_handle: tauri::AppHandle) -> PathBuf {
    // Attempt to get the "screentap_app_data_dir" environment variable
    let app_data_dir = match env::var("SCREENTAP_APP_DATA_DIR") {
        Ok(value) => {
            // If the environment variable exists, use its value
            Path::new(&value).to_path_buf()
        },
        Err(_) => {
            // If the environment variable does not exist, fall back to the default app data directory
            app_handle.path_resolver().app_data_dir().expect("Failed to get app_data_dir")
        }
    };
    app_data_dir
}

fn setup_handler(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error + 'static>> {

    let app_handle = app.handle();
    let db_filename_path = Path::new(DATABASE_FILENAME);

    // Attempt to get the "screentap_app_data_dir" environment variable
    // let app_data_dir = match env::var("SCREENTAP_APP_DATA_DIR") {
    //     Ok(value) => {
    //         // If the environment variable exists, use its value
    //         Path::new(&value).to_path_buf()
    //     },
    //     Err(_) => {
    //         // If the environment variable does not exist, fall back to the default app data directory
    //         app_handle.path_resolver().app_data_dir().expect("Failed to get app_data_dir")
    //     }
    // };
    let app_data_dir = get_effective_app_dir(app_handle);

    // let app_data_dir = app_handle.path_resolver().app_data_dir().expect("Failed to get app_data_dir");
    
    // If app_data_dir doesn't exist, create it
    if !app_data_dir.exists() {
        println!("Creating app_data_dir: {}", app_data_dir.as_path().to_str().unwrap());
        std::fs::create_dir_all(app_data_dir.as_path())?;
    } else {
        println!("Found existing app_data_dir: {}", app_data_dir.as_path().to_str().unwrap());
    }

    // Create the database if it doesn't exist
    match db::create_db(app_data_dir.as_path(), db_filename_path) {
        Ok(()) => (),
        Err(e) => eprintln!("Failed to create db: {}", e),
    }

    // Save one screenshot on startup so we never have an empty screen
    let _ = screenshot::save_screenshot(app_data_dir.as_path(), db_filename_path);

    // Create a compaction helper
    // TODO: still a WIP
    let compaction_helper = compaction::CompactionHelper::new(
        app_data_dir.clone(), 
        db_filename_path.to_path_buf(),
        compaction::DEFAULT_MAX_IMAGE_FILES,
    );

    // Spawn a thread to save screenshots in the background.
    // The move keyword is necessary to move app_data_dir into the thread.
    thread::spawn(move || {

        loop {

            // Compact screenshots to mp4 if necessary
            // TODO: enable this once its complete
            if compaction_helper.should_compact_screenshots() {
                let now = Local::now().naive_utc();
                let timestamp_mp4_filename = utils::generate_filename(now, "mp4");
                let timestamp_mp4_filename_fq = app_data_dir.join(timestamp_mp4_filename);
                compaction_helper.compact_screenshots_to_mp4(PathBuf::from(timestamp_mp4_filename_fq));
            }

            let sleep_time_secs = 10;  // TODO: set back to 60
            thread::sleep(Duration::from_secs(sleep_time_secs));
            let _ = screenshot::save_screenshot(app_data_dir.as_path(), db_filename_path);
        }
    });

    // Maximize the main window
    match app.get_window("main") {
        Some(w) => {
            let maximize_result = w.maximize();
            match maximize_result {
                Ok(_) => {},
                Err(e) => {
                    println!("Error maximizing window: {}", e);
                }
            }
            _ = w.set_title("Screenstap: search");
        },
        None => {
            println!("Cannot get main window to maximize it");
        }
    }

    Ok(())

}


fn create_browse_screenshots_window(app: &tauri::AppHandle) -> tauri::Window {
 
    let new_window = tauri::WindowBuilder::new(
        app,
        "browse",
        tauri::WindowUrl::App("index_browse.html".into())
    ).maximized(true).title("Screentap: browse").build().expect("failed to build window");

    new_window
}

fn handle_system_tray_event(app: &tauri::AppHandle, event: tauri::SystemTrayEvent) {

    if let SystemTrayEvent::MenuItemClick{ id, .. } = event {
        match id.as_str() {
            "quit" => {
                std::process::exit(0);
            },
            "search" => {

                let window = app.get_window("main");
                match window { 
                    Some(w) => {
                        w.show().unwrap();
                        w.set_focus().unwrap();
                    },
                    None => {
                        let _ = tauri::WindowBuilder::new(
                            app,
                            "main",
                            tauri::WindowUrl::App("index.html".into())
                        ).maximized(true).title("Screentap: search").build().expect("failed to build window");
                    }
                }
            },
            "browse_screenshots" => {
                let window = app.get_window("browse");
                match window {
                    Some(w) => {
                        w.show().unwrap();
                        w.set_focus().unwrap();
                    },
                    None => {
                        let _ = create_browse_screenshots_window(app);
                    }
                }   
            },
            _ => {}
        }
    }

}


fn main() {

    println!("Starting screentap...");

    let quit = CustomMenuItem::new("quit".to_string(), "Quit").accelerator("Cmd+Q");
    let show_hide_window = CustomMenuItem::new("search".to_string(), "Search");
    let browse_screenshots_menu_item = CustomMenuItem::new("browse_screenshots".to_string(), "Browse");

    let system_tray_menu = SystemTrayMenu::new()
        .add_item(show_hide_window)
        .add_item(browse_screenshots_menu_item)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(quit);

    tauri::Builder::default()
    .setup(setup_handler)
    .system_tray(SystemTray::new().with_menu(system_tray_menu))
    .on_system_tray_event(handle_system_tray_event)
    .invoke_handler(tauri::generate_handler![
        search_screenshots, 
        browse_screenshots]
    )
    .run(tauri::generate_context!())
    .expect("Error while starting screentap");
    
}
