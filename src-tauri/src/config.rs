use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub mode: String,
    pub port: String,
    pub ip: String,
    pub was_running: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            mode: "lan".to_string(),
            port: "5000".to_string(),
            ip: String::new(),
            was_running: false,
        }
    }
}

impl Config {
    fn get_config_path() -> PathBuf {
        let config_dir = if cfg!(windows) {
            dirs::config_dir()
                .map(|p| p.join("WebInput"))
                .unwrap_or_else(|| PathBuf::from("."))
        } else {
            dirs::config_dir()
                .map(|p| p.join("webinput"))
                .unwrap_or_else(|| PathBuf::from("."))
        };

        fs::create_dir_all(&config_dir).ok();
        config_dir.join("config.json")
    }

    pub fn load() -> Self {
        let config_path = Self::get_config_path();

        if !config_path.exists() {
            return Self::default();
        }

        match fs::read_to_string(&config_path) {
            Ok(content) => {
                serde_json::from_str(&content).unwrap_or_default()
            }
            Err(_) => Self::default(),
        }
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_path = Self::get_config_path();
        let content = serde_json::to_string_pretty(self)?;
        fs::write(config_path, content)?;
        Ok(())
    }
}
