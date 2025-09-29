use std::fmt::Display;

use crate::config::Config;
use reqwest::{Client, StatusCode};
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

#[derive(Debug, Clone)]
pub enum JiraClientError {
    Request(String),
    Response(StatusCode, String),
    Parse,
}

impl Display for JiraClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JiraClientError::Request(err) => write!(f, "Jira Client request error: {}", err),
            JiraClientError::Response(status_code, error_text) => write!(
                f,
                "Bad response, status: {}, text: {}",
                status_code, error_text
            ),
            JiraClientError::Parse => write!(f, "Parse response error"),
        }
    }
}

impl JiraClient {
    pub fn new(config: &Config) -> Self {
        let client = Client::new();
        let auth_header = format!("Bearer {}", config.api_token);
        Self {
            client,
            config: config.clone(),
            auth_header,
        }
    }
}

pub async fn create_issue(
    jira_client: &JiraClient,
    project_key: &str,
    summary: &str,
    description: Option<&str>,
    issue_type_id: &str,
) -> Result<String, JiraClientError> {
    let api_url = format!(
        "{}/rest/api/2/issue",
        jira_client.config.jira_url.trim_end_matches('/')
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

    let response = jira_client
        .client
        .post(&api_url)
        .header("Authorization", &jira_client.auth_header)
        .header("Content-Type", "application/json")
        .json(&issue_data)
        .send()
        .await
        .map_err(|err| JiraClientError::Request(err.to_string()))?;

    if !response.status().is_success() {
        return Err(JiraClientError::Response(
            response.status(),
            response.text().await.unwrap_or_default(),
        ));
    }

    let create_response: CreateIssueResponse =
        response.json().await.map_err(|_| JiraClientError::Parse)?;

    // Возвращаем ссылку на созданную задачу
    Ok(jira_client.config.issue_url(&create_response.key))
}

pub async fn test_connection(client: &JiraClient) -> Result<(), JiraClientError> {
    let api_url = format!(
        "{}/rest/api/2/myself",
        client.config.jira_url.trim_end_matches('/')
    );

    let response = client
        .client
        .get(&api_url)
        .header("Authorization", &client.auth_header)
        .send()
        .await
        .map_err(|err| JiraClientError::Request(err.to_string()))?;

    if response.status().is_success() {
        Ok(())
    } else {
        Err(JiraClientError::Response(
            response.status(),
            response.text().await.unwrap_or_default(),
        ))
    }
}

pub async fn get_project_issue_types(
    jira_client: &JiraClient,
    project_key: &str,
) -> Result<Vec<IssueType>, JiraClientError> {
    let api_url = format!(
        "{}/rest/api/2/issue/createmeta/{}/issuetypes",
        jira_client.config.jira_url.trim_end_matches('/'),
        project_key
    );

    let response = jira_client
        .client
        .get(&api_url)
        .header("Authorization", &jira_client.auth_header)
        .send()
        .await
        .map_err(|err| JiraClientError::Request(err.to_string()))?;

    if !response.status().is_success() {
        return Err(JiraClientError::Response(
            response.status(),
            response.text().await.unwrap_or_default(),
        ));
    }

    let issue_types_response: IssueTypesResponse =
        response.json().await.map_err(|_| JiraClientError::Parse)?;
    Ok(issue_types_response.values)
}
