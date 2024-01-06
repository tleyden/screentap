use swift_rs::{swift, SRString};

swift!(fn perform_ocr_swift(path: &SRString) -> Option<SRString>);

pub fn extract_text(path: &str) -> String {
    let value: SRString = path.into();
    let result = unsafe { perform_ocr_swift(&value) };
    String::from(result.unwrap().as_str())
}
