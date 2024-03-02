use std::path::PathBuf;
use chrono::NaiveDateTime;
use url::Url;


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
            set theURL to URL of active tab of front window
            return theURL
        end tell
    "#;
    execute_applescript(script)
}

pub fn get_safari_browser_tab_name() -> String {
    let script = r#"
        tell application "Safari"
            set theURL to URL of front document
            return theURL
        end tell
    "#;
    execute_applescript(script)

}

/**
 * Has the frontmost app or browser tab changed?
 * 
 * If the frontmost app has changed, return true.
 * Otherwise, if the frontmost app is a browser, check if the tab has changed
 * 
 * TODO: move this logic and state into focusguard 
 */
pub fn frontmost_app_or_browser_tab_changed(cur_frontmost_app: &str, last_frontmost_app: &str, cur_browser_tab_url: &str, last_browser_tab_url: &str) -> bool {

    if cur_frontmost_app != last_frontmost_app {
        true
    } else if cur_frontmost_app == "com.google.Chrome" || cur_frontmost_app == "com.apple.Safari" {
        // Normalize the browser urls into domain names since a user might be opening multiple tabs on
        // a distracting site like reddit.  Without this normalization, each web page on reddit would
        // be considered a different context, but in fact we want each reddit tab to be considered the
        // same context of the user surfing reddit.
        let cur_url_domain = extract_domain_from_url(cur_browser_tab_url).unwrap_or("".to_string());
        let last_url_domain = extract_domain_from_url(last_browser_tab_url).unwrap_or("".to_string());

        cur_url_domain != last_url_domain
    } else {
        false
    }

}


fn extract_domain_from_url(url: &str) -> Result<String, &'static str> {
    let parsed_url = Url::parse(url).map_err(|_| "Invalid URL")?;
    parsed_url.host_str().map(|s| s.to_string()).ok_or("Domain not found")
}
