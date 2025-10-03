use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LoadConfigError {
    #[error("Failed to read config file")]
    Read,
    #[error("Failed to deserialize config file")]
    Deserialize,
}

#[derive(Debug, Error)]
pub enum SaveConfigError {
    #[error("Failed to create config directory")]
    CreateDir,
    #[error("Failed to serialize config")]
    Serialize,
    #[error("Failed to save config file")]
    Save,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct Config {
    pub jira_url: String,
    pub email: String,
    pub api_token: String,
    pub projects: HashMap<String, String>,
}

pub static CONFIG_PATH: Lazy<PathBuf> = Lazy::new(|| {
    let mut path = dirs::config_dir().expect("Could not find config directory");
    path.push("fast-task");
    path.push("config.json");
    path
});

pub fn load_config() -> Result<Config, LoadConfigError> {
    let content = fs::read_to_string(CONFIG_PATH.as_path()).map_err(|_| LoadConfigError::Read)?;
    let config: Config =
        serde_json::from_str(&content).map_err(|_| LoadConfigError::Deserialize)?;
    Ok(config)
}

pub fn save_config(config: Config) -> Result<(), SaveConfigError> {
    if let Some(parent) = CONFIG_PATH.parent() {
        fs::create_dir_all(parent).map_err(|_| SaveConfigError::CreateDir)?;
    }
    let content = serde_json::to_string_pretty(&config).map_err(|_| SaveConfigError::Serialize)?;
    fs::write(CONFIG_PATH.as_path(), content).map_err(|_| SaveConfigError::Save)?;
    Ok(())
}

impl Config {
    pub fn new(
        jira_url: String,
        email: String,
        api_token: String,
        projects: HashMap<String, String>,
    ) -> Self {
        Config {
            jira_url,
            email,
            api_token,
            projects,
        }
    }

    pub fn is_configured(&self) -> bool {
        !self.jira_url.is_empty() && !self.email.is_empty() && !self.api_token.is_empty()
    }

    pub fn issue_url(&self, issue_key: &str) -> String {
        format!(
            "{}/browse/{}",
            self.jira_url.trim_end_matches('/'),
            issue_key
        )
    }
}
