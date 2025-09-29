use core::panic;

use clap::{Parser, Subcommand};
use inquire::{Confirm, Select, Text};
use once_cell::sync::Lazy;
use validator::{ValidateEmail, ValidateUrl};

mod config;
mod jira_client;

use config::Config;
use jira_client::JiraClient;

use crate::config::{CONFIG_PATH, LoadConfigError, load_config, save_config};

#[derive(Parser)]
#[command(name = "fast-task")]
#[command(about = "A CLI tool for creating Jira issues")]
#[command(long_about = "Create Jira issues quickly from the command line.
Use 'fast-task create' for guided issue creation")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Configure Jira connection settings
    Config,
    /// Add a project to work with
    AddProject,
    /// List configured projects
    ListProjects,
    /// Test Jira connection
    Test,
    /// Create a new issue
    Create,
}

#[derive(Debug)]
enum IssueCreateError {
    JiraClient(String, String),
    EmptyTitle,
    IssueTypesNotFound(String),
    SelectOption,
    Canceled,
}

impl std::fmt::Display for IssueCreateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IssueCreateError::JiraClient(selected_project, msg) => {
                write!(
                    f,
                    "Project: {}. Jira client error: {}",
                    selected_project, msg
                )
            }
            IssueCreateError::EmptyTitle => write!(f, "Issue title cannot be empty"),
            IssueCreateError::IssueTypesNotFound(project) => {
                write!(f, "No issue types found for project '{}'", project)
            }
            IssueCreateError::SelectOption => write!(f, "Failed to select an option"),
            IssueCreateError::Canceled => write!(f, "Operation canceled by user"),
        }
    }
}

#[tokio::main]
async fn main() {
    Lazy::force(&CONFIG_PATH);
    let cli = Cli::parse();
    let config = match load_config() {
        Ok(config) => config,
        Err(LoadConfigError::Read) => {
            println!("Config read error, will use default config");
            Config::default()
        }
        Err(LoadConfigError::Deserialize) => {
            panic!("Cannot deserialize config file!");
        }
    };

    match cli.command {
        Commands::Config => interactive_set_config(&config),
        Commands::AddProject => interactive_add_project(&config),
        Commands::ListProjects => {
            let config = load_config().unwrap_or_default();
            if config.projects.is_empty() {
                println!("No projects configured. Use 'fast-task add-project' to add one.");
            } else {
                println!("Configured projects:");
                for (key, name) in &config.projects {
                    println!("  {} - {}", key, name);
                }
            }
        }

        Commands::Test => {
            if !config.is_configured() {
                println!("❌ Please configure Jira connection first:");
                println!("fast-task config ");
            }

            println!("🔍 Testing Jira connection...");
            let client = JiraClient::new(&config);
            match client.test_connection().await {
                Ok(_) => {
                    println!("✅ Connection successful!");
                    println!("   URL: {}", config.jira_url);
                    println!("   Email: {}", config.email);
                }
                Err(e) => {
                    println!("❌ Connection failed: {}", e);
                    println!("💡 Check your configuration:");
                    println!("   - URL: {}", config.jira_url);
                    println!("   - Email: {}", config.email);
                }
            }
        }

        Commands::Create => {
            if !config.is_configured() {
                println!("❌ Please configure Jira connection first:");
                println!("fast-task config");
            }

            if config.projects.is_empty() {
                println!("❌ No projects configured. Add one first:");
                println!("fast-task add-project <KEY> --name <NAME>");
            }

            match interactive_create_issue(&config).await {
                Ok(issue_url) => {
                    println!("✅ Issue created successfully!");
                    println!("🔗 {}", issue_url);
                }
                Err(e) => {
                    println!("❌ Failed to create issue: {:?}", e);
                }
            }
        }
    }
}

fn interactive_set_config(original_config: &Config) {
    println!("🎯 Setup a jira configuration\n");

    let mut jira_url: String;
    let mut email: String;
    let mut api_token: String;

    loop {
        jira_url = Text::new("Jira URL:")
            .with_help_message("Enter your Jira instance URL (include https://)")
            .with_placeholder("e.g., https://company.atlassian.net")
            .prompt()
            .expect("Cannot prompt");

        if !jira_url.validate_url() {
            println!("❌ jira url is not valid. Try again");
            continue;
        }
        break;
    }

    loop {
        email = Text::new("Your Jira email:")
            .with_help_message("Enter your email address for Jira authentication")
            .with_placeholder("user@company.com")
            .prompt()
            .expect("Cannot prompt");

        if !email.validate_email() {
            println!("❌ Email is not valid. Try again");
            continue;
        }
        break;
    }
    loop {
        api_token = Text::new("Your Jira api token:")
            .with_help_message("Enter your api token")
            .prompt()
            .expect("Cannot prompt");

        if api_token.trim().is_empty() {
            println!("❌ Api token cannot be empty. Try again");
            continue;
        }
        break;
    }
    let config = Config::new(jira_url, email, api_token, original_config.projects.clone());
    match save_config(config) {
        Ok(_) => {
            println!("Configuration saved!");
        }
        Err(err) => {
            println!("Failed to save config: {:?}", err);
        }
    }
}

fn interactive_add_project(original_config: &Config) {
    let mut project_key: String;
    let mut project_name: String;

    loop {
        project_key = Text::new("Your project key:")
            .with_help_message("Enter your project key")
            .with_placeholder("e.g. PRKEY")
            .prompt()
            .expect("Cannot prompt");

        if project_key.trim().is_empty() {
            println!("❌ Project key cannot be empty. Try again");
            continue;
        }
        break;
    }
    loop {
        project_name = Text::new("Your Jira project name:")
            .with_help_message("Enter name of your project (for display)")
            .prompt()
            .expect("Cannot prompt");

        if project_name.trim().is_empty() {
            println!("❌ Project name cannot be empty. Try again");
            continue;
        }
        break;
    }
    let mut projects = original_config.projects.clone();
    projects.insert(project_key, project_name);
    match save_config(Config::new(
        original_config.jira_url.clone(),
        original_config.email.clone(),
        original_config.api_token.clone(),
        projects,
    )) {
        Ok(_) => {
            println!("Configuration saved!");
        }
        Err(err) => {
            println!("Failed to save config: {:?}", err);
        }
    }
}

async fn interactive_create_issue(config: &Config) -> Result<String, IssueCreateError> {
    println!("🎯 Creating a new Jira issue \n");

    let client = JiraClient::new(config);
    let project_options: Vec<String> = config.projects.keys().cloned().collect();
    let selected_project = Select::new("Which project?", project_options)
        .with_help_message("Select the project where you want to create the issue")
        .prompt()
        .expect("Cannot prompt");

    println!(
        "✓ Selected project: {} ({})",
        selected_project,
        config
            .projects
            .get(&selected_project)
            .unwrap_or(&selected_project)
    );

    let title = Text::new("Issue title:")
        .with_help_message("Enter a brief, descriptive title for your issue")
        .with_placeholder("e.g., Fix login button styling")
        .prompt()
        .expect("Cannot prompt");

    if title.trim().is_empty() {
        return Err(IssueCreateError::EmptyTitle);
    }

    let has_description = Confirm::new("Add description?")
        .with_default(false)
        .with_help_message("Press 'y' to add a detailed description")
        .prompt()
        .expect("Cannot prompt");

    let description = if has_description {
        let desc = Text::new("Issue description:")
            .with_help_message("Provide detailed information about the issue")
            .with_placeholder("Steps to reproduce, expected behavior, etc.")
            .prompt()
            .expect("Cannot prompt");

        if desc.trim().is_empty() {
            None
        } else {
            Some(desc)
        }
    } else {
        None
    };

    println!(
        "🔍 Fetching available issue types for project {}...",
        selected_project
    );

    let issue_types = match client.get_project_issue_types(&selected_project).await {
        Ok(types) => {
            if types.is_empty() {
                return Err(IssueCreateError::IssueTypesNotFound(selected_project));
            } else {
                println!(
                    "✅ Found {} issue type(s) for project {selected_project}",
                    types.len()
                );
                types
            }
        }
        Err(e) => {
            return Err(IssueCreateError::JiraClient(
                selected_project,
                format!("Jira client error: {:?}", e),
            ));
        }
    };

    let issue_type_options: Vec<String> = issue_types
        .iter()
        .map(|it| {
            if let Some(ref description) = it.description {
                let desc = if description.len() > 60 {
                    format!("{}...", &description[..57])
                } else {
                    description.clone()
                };
                format!("{} - {}", it.name, desc)
            } else {
                it.name.clone()
            }
        })
        .collect();

    let selected_option = Select::new("Issue type:", issue_type_options.clone())
        .with_help_message("Select the type of issue you're creating")
        .prompt()
        .expect("Cannot prompt");

    let selected_index = issue_type_options
        .iter()
        .position(|option| option == &selected_option)
        .ok_or(IssueCreateError::SelectOption)?;

    let selected_issue_type = &issue_types[selected_index];

    println!("\n📋 Issue Summary:");
    println!(
        "   Project: {} ({})",
        selected_project,
        config
            .projects
            .get(&selected_project)
            .unwrap_or(&selected_project)
    );
    println!("   Title: {}", title);
    if let Some(ref desc) = description {
        println!(
            "   Description: {}",
            if desc.len() > 50 {
                format!("{}...", &desc[..50])
            } else {
                desc.clone()
            }
        );
    }
    println!("   Type: {}", selected_issue_type.name);
    if let Some(ref desc) = selected_issue_type.description {
        println!("   Type Description: {}", desc);
    }

    let confirm = Confirm::new("Create this issue?")
        .with_default(true)
        .prompt()
        .expect("Cannot prompt");

    if !confirm {
        return Err(IssueCreateError::Canceled);
    }
    println!("\n🚀 Creating issue...");
    Ok(client
        .create_issue(
            &selected_project,
            &title,
            description.as_deref(),
            selected_issue_type.id.as_str(),
        )
        .await
        .map_err(|e| {
            IssueCreateError::JiraClient(selected_option, format!("Jira client error: {:?}", e))
        }))?
}
