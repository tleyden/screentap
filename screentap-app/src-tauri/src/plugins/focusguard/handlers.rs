use tauri::Manager;
use rusqlite::params;
use chrono::Local;
use crate::plugins::focusguard::FocusGuard;
use crate::plugins::focusguard::config::FocusGuardConfig;
use std::path::PathBuf;
use std::path::Path;


#[tauri::command]
pub fn distraction_alert_rating(app_handle: tauri::AppHandle, liked: bool, screenshot_id: i64, png_image_path: &str, job_title: &str, job_role: &str) {

    println!("Distraction alert rating received: liked: {}, screenshot_id: {} png_image_path: {}", 
        liked, screenshot_id, png_image_path);

    let focus_guard_clone: tauri::State<Option<FocusGuard>> = app_handle.state();
    let focus_guard_ref = focus_guard_clone.as_ref().unwrap();

    let app_data_dir = &focus_guard_ref.app_data_dir;

    let target_image_path = copy_image_to_distraction_alerts_screenshots_dir(
        app_data_dir, 
        png_image_path, 
        screenshot_id
    );

    let screentab_db_path = &focus_guard_ref.screentap_db_path;

    let user_rating = if liked { 1 } else { 0 };

    insert_distraction_alert_record(
        screentab_db_path, 
        screenshot_id, 
        user_rating, 
        target_image_path.to_str().unwrap(), 
        job_title, 
        job_role
    );

}


fn insert_distraction_alert_record(screentap_db_path: &PathBuf, screenshot_id: i64, user_rating: i32, file_path: &str, job_title: &str, job_role: &str) {

    let conn = crate::plugins::focusguard::FocusGuard::get_db_conn(screentap_db_path);

    let now = Local::now().naive_utc();

    let result = conn.execute(
        "INSERT INTO focusguard_distraction_alerts (timestamp, screenshot_id, user_rating, file_path, job_title, job_role) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![now.timestamp(), screenshot_id, user_rating, file_path, job_title, job_role],
    );
    match result {
        Ok(_) => println!("Inserted new record into focusguard_distraction_alerts"),
        Err(e) => println!("Error inserting new record into focusguard_distraction_alerts: {:?}", e),
    }

}

fn copy_image_to_distraction_alerts_screenshots_dir(app_data_dir: &Path, png_image_path: &str, screenshot_id: i64) -> std::path::PathBuf {
 
    // Copy the image file to a specific location so it doesn't get compacted into an mp4
    let focusguard_root_dir = FocusGuardConfig::get_focusguard_root_dir(app_data_dir);

    // Is there a distraction alert screenshots dir?  If not, create it
    let distraction_alerts_screenshots_dir = focusguard_root_dir.join("distraction_alert_screenshots");
    if !distraction_alerts_screenshots_dir.exists() {
        std::fs::create_dir_all(&distraction_alerts_screenshots_dir).expect("Failed to create distraction_alerts_screenshots_dir");
    }

    // Get the filename part of the png_image_path
    let png_image_path = std::path::Path::new(png_image_path);
    let png_image_filename = png_image_path.file_name().unwrap();

    // Copy the image to the distraction_alert_screenshots dir
    let target_image_path = distraction_alerts_screenshots_dir.join(format!("{}_{}.png", screenshot_id, png_image_filename.to_str().unwrap()));
    std::fs::copy(png_image_path, &target_image_path).expect("Failed to copy image to distraction_alerts_screenshots_dir");

    target_image_path
}
