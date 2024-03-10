use swift_rs::{swift, SRString, SRData, Int, Bool, Float};

use std::fs::File;
use std::io::prelude::*;
use std::io::BufWriter;

swift!(fn perform_ocr_swift(path: &SRString) -> Option<SRString>);
swift!(fn screen_capture_swift() -> Option<SRData>);    
swift!(fn write_images_in_dir_to_mp4_swift(directory_path: &SRString, target_filename: &SRString, use_bitrate_key: Bool) -> ());
swift!(fn extract_frame_from_mp4_swift(mp4_path: &SRString, frame_id: Int) -> Option<SRData>);    
swift!(fn get_frontmost_app_swift() -> SRString);
swift!(fn resize_image_swift(image: SRData, scale: Float) ->  Option<SRData>);


pub fn extract_frame_from_mp4(mp4_path: &str, frame_id: isize) -> Option<SRData> {
    let mp4_path_str: SRString = mp4_path.into();
    // let frame_id_int: Int = frame_id.into();
    let result = unsafe { extract_frame_from_mp4_swift(&mp4_path_str, frame_id) };
    result
}


pub fn resize_image(png_data: Vec<u8>, scale: f32) -> Vec<u8> {

    // Convert the vector to a SRData
    let image = SRData::from(&*png_data);

    let result = unsafe { resize_image_swift(image, scale) };
    
    result.unwrap().to_vec()

}

/**
 * Given a path to a directory of images, write them to an mp4
 */
pub fn write_images_in_dir_to_mp4(directory_path: &str, target_filename: &str, use_bitrate_key: bool) -> () {
    let dirpath_str: SRString = directory_path.into();
    let target_filename_str: SRString = target_filename.into();
    
    println!(
        "Writing images to mp4 in: {} to: {} with use_bitrate_key: {}", 
        dirpath_str.as_str(), 
        target_filename_str.as_str(), 
        use_bitrate_key
    );
    
    unsafe { 
        write_images_in_dir_to_mp4_swift(
            &dirpath_str, 
            &target_filename_str, 
            use_bitrate_key
        ) 
    };
    
}


/**
 * Given a path to an image, extract the text from it using OCR
 */
pub fn extract_text(path: &str) -> String {
    let value: SRString = path.into();
    let result = unsafe { perform_ocr_swift(&value) };
    String::from(result.unwrap().as_str())
}

/**
 * Get the name of the frontmost app
 * 
 * NOTE: no longer used because it was returning stale values, and using KVO observing
 * or NSWorkspace.DidActivateApplicationNotification appears to be difficult to do
 * with the swift-rs bridge.
 */
pub fn get_frontmost_app() -> String {
    let result = unsafe { get_frontmost_app_swift() };
    result.to_string()
}


/**
 * Capture the screen and write to a file
 */
pub fn screen_capture_to_file(dest_file: &str) -> Vec<u8> {
    let png_sr_data: Option<SRData> = unsafe { screen_capture_swift() };
    let png_data = png_sr_data.unwrap().to_vec();
    // Print the length of the vector
    let _ = write_png_to_file(&png_data, dest_file);
    png_data
}

/**
 * Capture the screen and return the raw image data
 */
pub fn screen_capture() -> Option<SRData> {
    let result = unsafe { screen_capture_swift() };
    result
}


/**
 * Helper function to write a PNG to a file
 */
fn write_png_to_file(image_data: &Vec<u8>, file_path: &str) -> std::io::Result<()> {
    // Create a file in write-only mode
    let file = File::create(file_path)?;
    let mut buf_writer = BufWriter::new(file);

    // Write the byte array to the file
    buf_writer.write_all(image_data)?;

    Ok(())
}

