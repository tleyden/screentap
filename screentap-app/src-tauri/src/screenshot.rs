extern crate screen_ocr_swift_rs;

use chrono::Local;
use std::path::Path;

use super::utils;
use super::db;


/**
 * Helper function to save a screenshot and OCR text to the dataset directory and DB
 */
pub fn save_screenshot(dataset_root: &str, db_filename: &str) -> String {

    let now = Local::now().naive_utc();

    let timestamp_png_filename = utils::generate_filename(now, "png");
    let dataset_root_path = Path::new(dataset_root);
    let target_png_file_path = dataset_root_path.join(timestamp_png_filename.clone());

    screen_ocr_swift_rs::screen_capture(target_png_file_path.to_str().unwrap());
    let ocr_text = screen_ocr_swift_rs::extract_text(target_png_file_path.to_str().unwrap());

    // When saving to the db, use the file relative to the public directory so that it can be served by the web server
    let relative_png_file_path = format!("dataset/{}", timestamp_png_filename.clone().to_str().unwrap());
    
    // Save screenshot meta to the DB
    let save_result = db::save_screenshot_meta(
        relative_png_file_path.as_str(), 
        ocr_text.to_string().as_str(),
        dataset_root,
        db_filename,
        now
    );

    let current_time_formatted = now.format("%Y-%m-%d %H:%M:%S").to_string();
    let result = match save_result {
        Ok(()) => format!("Screenshot saved to DB successfully at {}", current_time_formatted),
        Err(e) => format!("Error occurred: {} at {}", e, current_time_formatted),
    };

    result  // TODO: return the save_result instead of this string
}
