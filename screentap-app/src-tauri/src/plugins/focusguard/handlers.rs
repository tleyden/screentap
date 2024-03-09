use tauri::Manager;
use rusqlite::params;
use chrono::Local;

#[tauri::command]
pub fn distraction_alert_rating(app_handle: tauri::AppHandle, liked: bool, screenshot_id: i64, png_image_path: &str, job_title: &str, job_role: &str) -> () {

    println!("Distraction alert rating received: liked: {}, screenshot_id: {} png_image_path: {} job_title: {} job_role: {}", 
        liked, screenshot_id, png_image_path, job_title, job_role);

    let focus_guard_clone: tauri::State<Option<crate::plugins::focusguard::FocusGuard>> = app_handle.state();

    let focus_guard_ref = focus_guard_clone.as_ref().unwrap();

    println!("focus_guard_clone.screentap_db_path: {:?}", focus_guard_ref.screentap_db_path);

    let conn = crate::plugins::focusguard::FocusGuard::get_db_conn(&focus_guard_ref.screentap_db_path);

    // Copy the image file to a specific location so it doesn't get compacted into an mp4

    // Insert a new record into the DB, using the dataset dir 

    let user_rating = if liked { 1 } else { 0 };

    let now = Local::now().naive_utc();

    let result = conn.execute(
        "INSERT INTO focusguard_distraction_alerts (timestamp, screenshot_id, user_rating, file_path, job_title, job_role) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![now.timestamp(), screenshot_id, user_rating, png_image_path, job_title, job_role],
    );
    match result {
        Ok(_) => println!("Inserted new record into focusguard_distraction_alerts"),
        Err(e) => println!("Error inserting new record into focusguard_distraction_alerts: {:?}", e),
    }

    // Figure out the path to the DB by calling a static method

    // Open the DB connection

    // Write a new record to the DB

    // Get the job role and description used for inference on the LLM, to help measure prompt engineering
    



}