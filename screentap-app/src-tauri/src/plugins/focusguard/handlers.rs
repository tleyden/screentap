
#[tauri::command]
// pub fn distraction_alert_rating(app_handle: tauri::AppHandle, liked: bool, productivity_score: i32, raw_llm_result: &str, screenshot_id: i64) -> () {
pub fn distraction_alert_rating(app_handle: tauri::AppHandle, liked: bool, screenshot_id: i64, png_image_path: &str) -> () {

    println!("Distraction alert rating received: liked: {}, screenshot_id: {} png_image_path: {}", liked, screenshot_id, png_image_path);

    // Figure out the path to the DB by calling a static method

    // Open the DB connection

    // Write a new record to the DB

    // Copy the image file to a specific location



}