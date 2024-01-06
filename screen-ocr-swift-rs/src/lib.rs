use swift_rs::{swift, SRString, SRData};

swift!(fn perform_ocr_swift(path: &SRString) -> Option<SRString>);
swift!(fn screen_capture_swift() -> Option<SRData>);

pub fn extract_text(path: &str) -> String {
    let value: SRString = path.into();
    let result = unsafe { perform_ocr_swift(&value) };
    String::from(result.unwrap().as_str())
}

pub fn screen_capture() -> () {
    let result = unsafe { screen_capture_swift() };
    let result_vec = result.unwrap().to_vec();
    // Print the length of the vector
    println!("Length of vector: {}", result_vec.len());
}
