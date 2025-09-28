use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct Config {
    pub jira_url: String,
    pub email: String,
    pub api_token: String,
    pub projects: HashMap<String, String>,
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = Self::get_config_path()?;

        if !config_path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(&config_path)
            .with_context(|| format!("Failed to read config from {:?}", config_path))?;

        let config: Config =
            serde_json::from_str(&content).with_context(|| "Failed to parse config JSON")?;

        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let config_path = Self::get_config_path()?;

        // Создаем директорию если её нет
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config directory {:?}", parent))?;
        }

        let content =
            serde_json::to_string_pretty(self).with_context(|| "Failed to serialize config")?;

        fs::write(&config_path, content)
            .with_context(|| format!("Failed to write config to {:?}", config_path))?;

        Ok(())
    }

    pub fn set_data(&mut self, jira_url: String, email: String, api_token: String) -> Result<()> {
        self.jira_url = jira_url;
        self.email = email;
        self.api_token = api_token;
        self.save()?;
        Ok(())
    }

    fn get_config_path() -> Result<PathBuf> {
        let mut path =
            dirs::config_dir().ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?;

        path.push("fast-task");
        path.push("config.json");

        Ok(path)
    }

    pub fn is_configured(&self) -> bool {
        !self.jira_url.is_empty() && !self.email.is_empty() && !self.api_token.is_empty()
    }

    pub fn get_issue_url(&self, issue_key: &str) -> String {
        format!(
            "{}/browse/{}",
            self.jira_url.trim_end_matches('/'),
            issue_key
        )
    }
}
