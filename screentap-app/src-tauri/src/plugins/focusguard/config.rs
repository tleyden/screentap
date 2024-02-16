use std::path::Path;
use toml;
use std::fs;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct FocusGuardConfig {
    pub job_title: String,
    pub job_role: String,
    pub duration_between_checks_secs: u64,
    pub duration_between_alerts_secs: u64,
    pub llava_backend: String,
    pub productivity_score_threshold: i32,
    pub image_dimension_longest_side: u32,
}

impl FocusGuardConfig {

    pub fn new(app_data_dir: &Path) -> Option<FocusGuardConfig> {

        // Build path to config.toml in expected place
        let toml_config = app_data_dir
            .join("plugins")
            .join("focusguard")
            .join("config.toml");

        // If config.toml not found, return None
        if !toml_config.exists() {
            println!("FocusGuard config not found at path: {}", toml_config.display());
            return None;
        }

        let config_str = fs::read_to_string(toml_config)
            .expect("Failed to read focusguard config file");

        let focusguard_config = toml::from_str::<FocusGuardConfig>(&config_str)
            .expect("Failed to parse focusguard config");

        Some(focusguard_config)

    }
}