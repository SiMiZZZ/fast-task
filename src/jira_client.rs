use thiserror::Error;

use crate::config::Config;
use crate::jql::{self, Field, JqlBuilder, SortOrder};
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize, Deserialize, Debug)]
pub struct SearchResponse {
    #[serde(rename = "startAt")]
    pub start_at: u16,
    #[serde(rename = "maxResults")]
    pub max_results: u16,
    pub total: u16,
    pub issues: Vec<Issue>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Issue {
    pub key: String,
    pub fields: IssueFields,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct IssueFields {
    pub summary: String,
    pub status: IssueStatus,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct IssueStatus {
    pub name: String,
    #[serde(rename = "statusCategory")]
    pub status_category: IssueStatusCategory,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct IssueStatusCategory {
    pub key: String,
}

#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub enum IssueStatusFilter {
    /// Exclude done/resolved issues (default behavior).
    #[default]
    ActiveOnly,
    /// Include all issues regardless of status.
    All,
    /// Show only issues with this specific status name.
    Only(String),
}

#[derive(Debug, Clone, Default)]
pub struct MyIssuesQuery {
    pub status_filter: IssueStatusFilter,
    pub project: Option<String>,
}

const SEARCH_PAGE_SIZE: u16 = 50;

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

#[derive(Debug, Error)]
pub enum JiraClientError {
    #[error("Jira Client request error: {0}")]
    Request(String),
    #[error("Bad response, status: {0}, text: {1}")]
    Response(StatusCode, String),
    #[error("Parse response error")]
    Parse,
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

pub async fn get_my_issues(
    jira_client: &JiraClient,
    query: &MyIssuesQuery,
) -> Result<Vec<Issue>, JiraClientError> {
    let mut builder = JqlBuilder::new().assignee_is_current_user();
    match &query.status_filter {
        IssueStatusFilter::ActiveOnly => {
            builder = builder.exclude_done();
        }
        IssueStatusFilter::All => {}
        IssueStatusFilter::Only(status) => {
            builder = builder.and(
                Field::Status,
                jql::Operator::Eq,
                jql::Value::Str(status.clone()),
            );
        }
    }
    if let Some(project_key) = &query.project {
        builder = builder
            .project(project_key)
            .map_err(|e| JiraClientError::Request(e.to_string()))?;
    }
    let jql = builder.order_by(Field::Updated, SortOrder::Desc).build();

    let api_url = format!(
        "{}/rest/api/2/search",
        jira_client.config.jira_url.trim_end_matches('/')
    );

    let mut all_issues = Vec::new();
    let max_results = SEARCH_PAGE_SIZE;
    let mut start_at = 0;

    loop {
        let body = json!({
            "jql": jql,
            "startAt": start_at,
            "maxResults": max_results,
            "fields": ["summary", "status"]
        });

        let response = jira_client
            .client
            .post(&api_url)
            .header("Authorization", &jira_client.auth_header)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|err| JiraClientError::Request(err.to_string()))?;

        if !response.status().is_success() {
            return Err(JiraClientError::Response(
                response.status(),
                response.text().await.unwrap_or_default(),
            ));
        }

        let search_response: SearchResponse =
            response.json().await.map_err(|_| JiraClientError::Parse)?;

        let fetched = search_response.issues.len() as u16;
        all_issues.extend(search_response.issues);
        start_at += fetched;

        if start_at >= search_response.total || fetched == 0 {
            break;
        }
    }

    Ok(all_issues)
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
