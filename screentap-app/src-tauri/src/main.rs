// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

extern crate screen_ocr_swift_rs;

use chrono::NaiveDateTime;
use tauri::NativeImage;
use tauri::{Manager, SystemTray, SystemTrayEvent, SystemTrayMenu, CustomMenuItem, SystemTrayMenuItem};

use std::collections::HashMap;
use std::thread;
use std::env;
use std::time::Duration;
use std::path::Path;
use std::path::PathBuf;
use chrono::Local;
use crate::plugins::focusguard;

mod db;
mod utils; 
mod screenshot;
mod compaction;
mod plugins;


static DATABASE_FILENAME: &str = "screentap.db";


/**
 * How often to check if it's time to capture a screenshot based on the
 * the following logic:
 * 
 * - If the frontmost app changes, capture a screenshot
 * - If more than MAX_DURATION_BETWEEN_SCREEN_CAPTURES_SECS has passed since the last screenshot, capture a screenshot
 */
static DURATION_BETWEEN_SCREEN_CAPTURES_CHECKS: i64 = 5;

/**
 * The maximum duration between screenshots in seconds.  If the frontmost app changes frequently,
 * then more screenshots will be captured.  However if the frontmost app doesn't change, then
 * this timeout value will be used to trigger a screenshot capture.
 */
static MAX_DURATION_BETWEEN_SCREEN_CAPTURES_SECS: i64 = 60;

#[tauri::command]
fn search_screenshots(app_handle: tauri::AppHandle, term: &str) -> Vec<HashMap<String, String>> {

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

    let app_data_dir: PathBuf = get_effective_app_dir(app_handle);

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

    let app_data_dir = get_effective_app_dir(app_handle);
    
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
    let screenshot_result = screenshot::save_screenshot(app_data_dir.as_path(), db_filename_path);
    match screenshot_result {
        Ok(_) => {},
        Err(e) => {
            println!("Error saving screenshot on startup: {}", e);
        }
    }
    let mut last_screenshot_time = Local::now().naive_utc();
    let mut last_frontmost_app = screen_ocr_swift_rs::get_frontmost_app();

    let app_observer = screen_ocr_swift_rs::create_app_change_observer_rust();
    println!("created app_observer");

    // Create a compaction helper
    let compaction_helper = compaction::CompactionHelper::new(
        app_data_dir.clone(), 
        db_filename_path.to_path_buf(),
        compaction::DEFAULT_MAX_IMAGE_FILES,
    );

    // Create a focusguard instance
    let mut focus_guard_option = focusguard::FocusGuard::new_from_config(
        // Clone app_data_dir so focusguard can own the app data dir path instance
        // and we avoid reference lifetime issues
        // TODO: review this, it feels a bit overcomplicated
        PathBuf::from(app_data_dir.clone()),  
    );
    if focus_guard_option.is_none() {
        println!("FocusGuard not initialized");
    }


    // Get an app handle from the app since this can be moved to threads
    let app_handle = app.app_handle();

    // Spawn a thread to save screenshots in the background.
    // The move keyword is necessary to move app_data_dir into the thread.
    thread::spawn(move || {

        loop {

            let now = Local::now().naive_utc();

            // Compact screenshots to mp4 if necessary
            if compaction_helper.should_compact_screenshots() {
                let timestamp_mp4_filename = utils::generate_filename(now, "mp4");
                let timestamp_mp4_filename_fq = app_data_dir.join(timestamp_mp4_filename);

                compaction_helper.compact_screenshots_to_mp4(
                    timestamp_mp4_filename_fq, 
                    // TODO: make the use_bitrate_key a setting, otherwise it will
                    // crash on certain machines
                    false
                );
            }

            // Get the name of the frontmost app
            let cur_frontmost_app = screen_ocr_swift_rs::get_frontmost_app();

            // Check if it's time to capture a new screenshot based on whether the
            // frontmost app has changed or if it's been a while since the last screenshot
            let frontmost_app_changed = cur_frontmost_app != last_frontmost_app;
            println!("frontmost_app_changed: {} cur_frontmost_app: {} last_frontmost_app: {}", frontmost_app_changed, &cur_frontmost_app, last_frontmost_app);

            last_frontmost_app = cur_frontmost_app;

            let should_capture = should_capture_screenshot(last_screenshot_time, now, frontmost_app_changed);
            println!("Should_capture: {} last_screenshot_time: {} now: {}", should_capture, last_screenshot_time, now);

            if should_capture {

                // Update tracking variables
                last_screenshot_time = now;
                

                // Capture a screenshot, OCR and save it to DB
                let screenshot_result = screenshot::save_screenshot(app_data_dir.as_path(), db_filename_path);
                match screenshot_result {
                    Ok((png_data, ocr_text, png_image_path)) => {
                        // Invoke plugins
                        match focus_guard_option {

                            // TODO: any way to avoid this confusing "ref mut" stuff?
                            Some(ref mut focus_guard) => {
                                focus_guard.handle_screentap_event(
                                    &app_handle,
                                    png_data,
                                    png_image_path.as_path(),
                                    ocr_text,
                                    &last_frontmost_app,
                                    frontmost_app_changed
                                );        
                            },
                            None => ()
                        }
                    },
                    Err(e) => {
                        println!("Error saving screenshot: {}", e);
                    }
                }
            }






            thread::sleep(Duration::from_secs(DURATION_BETWEEN_SCREEN_CAPTURES_CHECKS as u64));

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


fn should_capture_screenshot(last_screenshot_time: NaiveDateTime, now: NaiveDateTime, frontmost_app_changed: bool) -> bool {

    let duration_since_last_screenshot = now.signed_duration_since(last_screenshot_time).num_seconds();

    if frontmost_app_changed {
        return true;
    } else if duration_since_last_screenshot > MAX_DURATION_BETWEEN_SCREEN_CAPTURES_SECS {
        return true;
    } else {
        return false;
    }
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
