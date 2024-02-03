use swift_rs::{swift, SRString, SRData};

use std::fs::File;
use std::io::prelude::*;
use std::io::BufWriter;


swift!(fn perform_ocr_swift(path: &SRString) -> Option<SRString>);
swift!(fn screen_capture_swift() -> Option<SRData>);    
swift!(fn cap_screenshot_to_mp4_swift(srdata: SRData) -> Option<SRString>);
swift!(fn write_images_in_dir_to_mp4_swift(directory_path: &SRString, target_filename: &SRString) -> ());

/**
 * Given a path to a directory of images, write them to an mp4
 */
pub fn write_images_in_dir_to_mp4(directory_path: &str, target_filename: &str) -> () {
    let dirpath_str: SRString = directory_path.into();
    let target_filename_str: SRString = target_filename.into();
    println!("Calling write_images_in_dir_to_mp4_swift with dirpath: {} and target_filename: {}", dirpath_str.as_str(), target_filename_str.as_str());
    unsafe { write_images_in_dir_to_mp4_swift(&dirpath_str, &target_filename_str) };
    println!("Finished call to write_images_in_dir_to_mp4_swift ");
}

pub fn cap_screenshot_to_mp4() -> String {

    let screen_capture_opt = unsafe { screen_capture_swift() };
    let screen_capture = screen_capture_opt.expect("Failed to get screen capture");

    // Create an array of SRData
    // let mut screen_capture_vec = Vec::new();
    // screen_capture_vec.push(screen_capture);
    // let screen_capture_array = screen_capture_vec.as_slice();

    // let screen_capture_array = SRArray::from_vec(vec![screen_capture]);



    let result = unsafe { cap_screenshot_to_mp4_swift(screen_capture) };
    String::from(result.unwrap().as_str())
}

/**
 * Given a path to an image, extract the text from it using OCR
 * 
 * TODO: just pass the raw image data to the swift function for OCR
 */
pub fn extract_text(path: &str) -> String {
    let value: SRString = path.into();
    let result = unsafe { perform_ocr_swift(&value) };
    String::from(result.unwrap().as_str())
}

/**
 * Capture the screen and write to a file
 */
pub fn screen_capture_to_file(dest_file: &str) -> () {
    let result = unsafe { screen_capture_swift() };
    let result_vec = result.unwrap().to_vec();
    // Print the length of the vector
    let _ = write_png_to_file(result_vec, dest_file);
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
fn write_png_to_file(image_data: Vec<u8>, file_path: &str) -> std::io::Result<()> {
    // Create a file in write-only mode
    let file = File::create(file_path)?;
    let mut buf_writer = BufWriter::new(file);

    // Write the byte array to the file
    buf_writer.write_all(&image_data)?;

    Ok(())
}

