use tauri::Manager;


#[tauri::command]
// pub fn distraction_alert_rating(app_handle: tauri::AppHandle, liked: bool, productivity_score: i32, raw_llm_result: &str, screenshot_id: i64) -> () {
pub fn distraction_alert_rating(app_handle: tauri::AppHandle, liked: bool, screenshot_id: i64, png_image_path: &str, job_title: &str, job_role: &str) -> () {

    println!("Distraction alert rating received: liked: {}, screenshot_id: {} png_image_path: {} job_title: {} job_role: {}", 
        liked, screenshot_id, png_image_path, job_title, job_role);

    let focus_guard_clone: tauri::State<Option<crate::plugins::focusguard::FocusGuard>> = app_handle.state();

    println!("focus_guard_clone.screentap_db_path: {:?}", focus_guard_clone.as_ref().unwrap().screentap_db_path);

    // Figure out the path to the DB by calling a static method

    // Open the DB connection

    // Write a new record to the DB

    // Copy the image file to a specific location

    // Get the job role and description used for inference on the LLM, to help measure prompt engineering
    



}