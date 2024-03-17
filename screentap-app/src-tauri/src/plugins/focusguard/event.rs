
use std::path::Path;
use tauri;
use std::fmt;

#[derive(Debug)]
pub struct FocusGuardCallbackEvent<'cb> {
    pub app: &'cb tauri::AppHandle, 
    pub png_data: &'cb Vec<u8>, 
    pub png_image_path: &'cb Path, 
    pub screenshot_id: i64, 
    pub ocr_text: String, 
    pub frontmost_app: &'cb str, 
    pub frontmost_browser_tab: &'cb str, 
    pub frontmost_app_or_tab_changed: bool
}


impl<'cb> fmt::Display for FocusGuardCallbackEvent<'cb> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {

        write!(f, "screenshot_id: {} len(ocr_text): {} len(png_data): {} frontmost app: {} frontmost browser tab: {} ", 
            self.screenshot_id, 
            self.ocr_text.len(),
            self.png_data.len(),
            self.frontmost_app,
            self.frontmost_browser_tab
        )
    }
}
