use crate::config::Config;
use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;

pub struct JiraClient {
    client: Client,
    config: Config,
    auth_header: String,
}

#[derive(Serialize, Deserialize)]
struct CreateIssueResponse {
    key: String,
    #[serde(rename = "self")]
    self_url: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct IssueTypesResponse {
    #[serde(rename = "maxResults")]
    pub max_results: i32,
    #[serde(rename = "startAt")]
    pub start_at: i32,
    pub total: i32,
    #[serde(rename = "isLast")]
    pub is_last: bool,
    pub values: Vec<IssueType>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IssueType {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
}

impl JiraClient {
    pub fn new(config: &Config) -> Result<Self> {
        let client = Client::new();

        let auth_header = format!("Bearer {}", config.api_token);

        Ok(Self {
            client,
            config: config.clone(),
            auth_header,
        })
    }

    pub async fn create_issue(
        &self,
        project_key: &str,
        summary: &str,
        description: Option<&str>,
        issue_type_id: &str,
    ) -> Result<String> {
        let api_url = format!(
            "{}/rest/api/2/issue",
            self.config.jira_url.trim_end_matches('/')
        );

        let description_content = description.unwrap_or("").to_string();

        let issue_data = json!({
            "fields": {
                "project": {
                    "key": project_key
                },
                "summary": summary,
                "description": description_content,
                "issuetype": {
                    "id": issue_type_id.to_string(),
                }
            }
        });

        let response = self
            .client
            .post(&api_url)
            .header("Authorization", &self.auth_header)
            .header("Content-Type", "application/json")
            .json(&issue_data)
            .send()
            .await
            .context("Failed to send request to Jira API")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "Jira API returned error {}: {}",
                status,
                error_text
            ));
        }

        let create_response: CreateIssueResponse = response
            .json()
            .await
            .context("Failed to parse Jira API response")?;

        // Возвращаем ссылку на созданную задачу
        Ok(self.config.get_issue_url(&create_response.key))
    }

    pub async fn get_project_issue_types(&self, project_key: &str) -> Result<Vec<IssueType>> {
        let api_url = format!(
            "{}/rest/api/2/issue/createmeta/{}/issuetypes",
            self.config.jira_url.trim_end_matches('/'),
            project_key
        );

        let response = self
            .client
            .get(&api_url)
            .header("Authorization", &self.auth_header)
            .send()
            .await
            .context("Failed to fetch project metadata")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "Jira API returned error {}: {}",
                status,
                error_text
            ));
        }

        let issue_types_response: IssueTypesResponse = response
            .json()
            .await
            .context("Failed to parse issue types response")?;

        Ok(issue_types_response.values)
    }

    pub async fn test_connection(&self) -> Result<()> {
        let api_url = format!(
            "{}/rest/api/2/myself",
            self.config.jira_url.trim_end_matches('/')
        );

        let response = self
            .client
            .get(&api_url)
            .header("Authorization", &self.auth_header)
            .send()
            .await
            .context("Failed to connect to Jira API")?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Authentication failed: HTTP {}",
                response.status()
            ))
        }
    }
}
