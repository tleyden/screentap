// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

extern crate screen_ocr_swift_rs;

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
 * Capture screenshots on a fixed schedule 
 */
static DURATION_BETWEEN_SCREEN_CAPTURES_CHECKS: i64 = 30;


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
                "exact" => cur_id,
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
    let (mut last_frontmost_app, mut last_browser_tab) = utils::get_frontmost_app_via_applescript();

    // Create a compaction helper
    let compaction_helper = compaction::CompactionHelper::new(
        app_data_dir.clone(), 
        db_filename_path.to_path_buf(),
        compaction::DEFAULT_MAX_IMAGE_FILES,
    );

    // Create a focusguard instance
    let screentap_db_filename_fq_path = app_data_dir.join(db_filename_path);
    let mut focus_guard_option = focusguard::FocusGuard::new_from_config(
        // Clone app_data_dir so focusguard can own the app data dir path instance
        // and we avoid reference lifetime issues
        // TODO: review this, it feels a bit overcomplicated
        app_data_dir.clone(),
        screentap_db_filename_fq_path,
    );

    if focus_guard_option.is_none() {
        println!("FocusGuard not initialized");
    } else {
        // Put a copy of the focusguard instance into the app managed state,
        // so we can at least access the configuration from handlers.
        // Why a clone?  If the original focusguard is moved into the managed
        // state, then the closure below passed to the thread will no longer work        
        app.manage(focus_guard_option.clone());
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

            // Get the name of the frontmost app and browser tab (if applicable)
            let (cur_frontmost_app, cur_browser_tab) = utils::get_frontmost_app_via_applescript();
            let frontmost_app_or_tab_changed = utils::frontmost_app_or_browser_tab_changed(&cur_frontmost_app, &last_frontmost_app, &cur_browser_tab, &last_browser_tab);
            println!("Capturing screenshot.  cur_frontmost_app: {} last_frontmost_app: {} cur_browser_tab: {}, last_browser_tab: {} frontmost_app_or_tab_changed: {} ", &cur_frontmost_app, last_frontmost_app, cur_browser_tab, last_browser_tab, frontmost_app_or_tab_changed);
            
            // Capture a screenshot, OCR and save it to DB
            let screenshot_result = screenshot::save_screenshot(app_data_dir.as_path(), db_filename_path);
            match screenshot_result {
                Ok(screenshot::ScreenshotSaveResult { png_data, ocr_text, png_image_path, screenshot_id}) => {

                    // Invoke plugins
                    // TODO: any way to avoid this confusing "ref mut" stuff?
                    if let Some(ref mut focus_guard) = focus_guard_option {
                        focus_guard.handle_screentap_event(
                            &app_handle,
                            png_data,
                            png_image_path.as_path(),
                            screenshot_id,
                            ocr_text,
                            &cur_frontmost_app,
                            &cur_browser_tab,
                            frontmost_app_or_tab_changed
                        );        
                    }
                },
                Err(e) => {
                    println!("Error saving screenshot: {}", e);
                }
            }

            // Update the last_ tracking variables to the current values
            last_frontmost_app = cur_frontmost_app;
            last_browser_tab = cur_browser_tab;

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
    .setup(|app| {
        setup_handler(app)
    })
    .system_tray(SystemTray::new().with_menu(system_tray_menu))
    .on_system_tray_event(handle_system_tray_event)
    .invoke_handler(tauri::generate_handler![
        search_screenshots, 
        browse_screenshots,
        focusguard::handlers::distraction_alert_rating]
    )
    .run(tauri::generate_context!())
    .expect("Error while starting screentap");
    
}
