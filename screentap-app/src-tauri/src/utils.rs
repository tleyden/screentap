use std::path::PathBuf;
use chrono::NaiveDateTime;


/**
 * Helper function to generate a filename based on the current time
 */
pub fn generate_filename(now: NaiveDateTime, extension: &str) -> PathBuf {

    let formatted_time = now.format("%Y_%m_%d_%H_%M_%S").to_string();
    let filename = format!("{}.{}", formatted_time, extension);
    PathBuf::from(filename)
}


pub fn execute_applescript(script: &str) -> String {
    let output = std::process::Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()
        .expect("Failed to execute osascript");
    let output_str = String::from_utf8_lossy(&output.stdout);
    output_str.trim().to_string()

}

/**
 * Get the name of the frontmost app via applescript.  This uses the bundle identifier
 * since it is more informative.  For example, just using the "name" will return "Electron"
 * for VSCode, while using the bundle identifier will return "com.microsoft.VSCode".
 */
pub fn get_frontmost_app_via_applescript() -> (String, String) {
    let script = r#"
    tell application "System Events" to tell (first process whose frontmost is true) to get the bundle identifier of it
    "#;
    let frontmost_app = execute_applescript(script);

    let browser_tab_name = if frontmost_app == "com.google.Chrome" {
        get_chrome_browser_tab_name()
    } else if frontmost_app == "com.apple.Safari" {
        get_safari_browser_tab_name()
    } else {
        "".to_string()
    };

    (frontmost_app, browser_tab_name)


}

pub fn get_chrome_browser_tab_name() -> String {
    let script = r#"
        tell application "Google Chrome"
            set theTitle to title of active tab of front window
            return theTitle
        end tell
    "#;
    execute_applescript(script)
}

pub fn get_safari_browser_tab_name() -> String {
    let script = r#"
        tell application "Safari"
            set theTitle to name of front document
            return theTitle
        end tell
    "#;
    execute_applescript(script)

}

/**
 * Has the frontmost app or browser tab changed?
 * 
 * If the frontmost app has changed, return true.
 * Otherwise, if the frontmost app is a browser, check if the tab has changed
 */
pub fn frontmost_app_or_browser_tab_changed(cur_frontmost_app: &str, last_frontmost_app: &str, cur_browser_tab: &str, last_browser_tab: &str) -> bool {

    // Special handlers for switching between screentap itself and other apps .. just ignore these
    // transitions for now since they are pure noise
    if cur_frontmost_app == "missing value" || last_frontmost_app == "missing value" {  // in yarn tauri dev mode, the bundle identifier will be this
        return false
    }

    // TODO: make sure this works.  Needs to be syncd with tauri config
    if cur_frontmost_app == "com.screentap-app.dev" || last_frontmost_app == "com.screentap-app.dev" {
        return false
    }

    if cur_frontmost_app != last_frontmost_app {
        return true
    }

    if cur_frontmost_app == "com.google.Chrome" || cur_frontmost_app == "com.apple.Safari" {
        return cur_browser_tab != last_browser_tab
    }

    false

}
