use swift_rs::{swift, SRString, SRData};

use std::fs::File;
use std::io::prelude::*;
use std::io::BufWriter;


swift!(fn perform_ocr_swift(path: &SRString) -> Option<SRString>);
swift!(fn screen_capture_swift() -> Option<SRData>);
swift!(fn cap_screenshot_to_mp4_swift() -> Option<SRString>);

pub fn cap_screenshot_to_mp4() -> String {
    let result = unsafe { cap_screenshot_to_mp4_swift() };
    String::from(result.unwrap().as_str())
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
 * Capture the screen and return a byte array
 */
pub fn screen_capture(dest_file: &str) -> () {
    let result = unsafe { screen_capture_swift() };
    let result_vec = result.unwrap().to_vec();
    // Print the length of the vector
    let _ = write_png_to_file(result_vec, dest_file);
    
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

