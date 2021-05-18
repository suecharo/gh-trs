use crate::git;
use crate::Scheme;

use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;

use anyhow::{anyhow, ensure};
use anyhow::{Context, Result};
use git::RepoUrl;
use reqwest;
use serde::{Deserialize, Serialize};
use serde_yaml;
use url::Url;

/// Priority:
///
/// 1. Command line options
/// 2. URL of the git repository in cwd
///
/// Output error if host is not github.com
pub fn resolve_repository_url(
    git: &str,
    cwd: &Path,
    remote: &str,
    opt_repo_url: &Option<String>,
    opt_scheme: &Scheme,
) -> Result<RepoUrl> {
    let repo_url = match opt_repo_url {
        Some(string_url) => RepoUrl::new(&string_url, opt_scheme)?,
        None => git::get_repo_url(git, cwd, remote, opt_scheme)?,
    };
    Ok(repo_url)
}

#[derive(Debug)]
pub struct CommitUser {
    pub name: String,
    pub email: String,
}

/// Priority:
///
/// 1. Command line options
/// 2. name and email of the git repository in cwd
pub fn resolve_commit_user(
    git: &str,
    cwd: &Path,
    opt_name: &Option<String>,
    opt_email: &Option<String>,
) -> Result<CommitUser> {
    let commit_user = CommitUser {
        name: match opt_name {
            Some(name) => name.to_string(),
            None => git::get_user_name(git, cwd)?,
        },
        email: match opt_email {
            Some(email) => email.to_string(),
            None => git::get_user_email(git, cwd)?,
        },
    };
    ensure!(
        commit_user.name != "" && commit_user.email != "",
        "Please set the name and email of the user to commit to."
    );
    Ok(commit_user)
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    tools: Vec<Tool>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Tool {
    url: Url,
    language_type: String,
    attachments: Option<Vec<Attachment>>,
    testing: Option<Testing>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Attachment {
    target: Option<String>,
    url: Url,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Testing {
    attachments: Vec<Attachment>,
}

pub fn load_config(config_file: &str) -> Result<Config> {
    let config_content = match Url::parse(config_file) {
        Ok(url) => {
            let response = reqwest::blocking::get(url.as_str())
                .with_context(|| format!("Failed to get from remote URL: {:?}", url.as_str()))?;
            ensure!(
                response.status().is_success(),
                format!("Failed to get from remote URL: {:?}", url)
            );
            response.text().context("Failed to decode response body.")?
        }
        Err(_) => {
            let config_file_path = Path::new(config_file)
                .canonicalize()
                .context("Failed to resolve config file path.")?;
            let mut reader =
                BufReader::new(File::open(config_file_path.as_path()).with_context(|| {
                    format!("Failed to open file: {:?}", config_file_path.as_path())
                })?);
            let mut content = String::new();
            reader.read_to_string(&mut content).with_context(|| {
                format!("Failed to read file: {:?}", config_file_path.as_path())
            })?;
            content
        }
    };
    let config: Config =
        serde_yaml::from_str(&config_content).context("Failed to deserialize config content.")?;
    Ok(config)
}

// fn validate_with_json_schema() -> Result<()> {}

pub fn repo_owner(repo_url: &RepoUrl) -> Result<String> {
    let path_segments = repo_url
        .https
        .path_segments()
        .ok_or(anyhow!("Failed to parse path in parsed URL."))?
        .collect::<Vec<&str>>();
    ensure!(
        path_segments.len() >= 2,
        "The path length of the repository URL is too short."
    );
    Ok(path_segments[0].to_string())
}

pub fn repo_name(repo_url: &RepoUrl) -> Result<String> {
    let path_segments = repo_url
        .https
        .path_segments()
        .ok_or(anyhow!("Failed to parse path in parsed URL."))?
        .collect::<Vec<&str>>();
    ensure!(
        path_segments.len() >= 2,
        "The path length of the repository URL is too short."
    );
    Ok(path_segments[1].to_string().replace(".git", ""))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    mod resolve_repository_url {
        use super::*;

        #[test]
        fn ok() {
            assert!(resolve_repository_url(
                "git",
                &env::current_dir().unwrap(),
                "origin",
                &Some("https://github.com/suecharo/gh-trs.git".to_string()),
                &Scheme::Https,
            )
            .is_ok());
            assert!(resolve_repository_url(
                "git",
                &env::current_dir().unwrap(),
                "origin",
                &None,
                &Scheme::Https,
            )
            .is_ok());
        }
    }

    mod repo_owner {
        use super::*;

        #[test]
        fn ok() {
            let repo_url =
                RepoUrl::new("https://github.com/suecharo/gh-trs.git", &Scheme::Https).unwrap();
            assert_eq!(repo_owner(&repo_url).unwrap(), "suecharo");
        }
    }

    mod repo_name {
        use super::*;

        #[test]
        fn ok() {
            let repo_url =
                RepoUrl::new("https://github.com/suecharo/gh-trs.git", &Scheme::Https).unwrap();
            assert_eq!(repo_name(&repo_url).unwrap(), "gh-trs");
        }
    }
}
