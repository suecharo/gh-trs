use crate::git;

use std::fs;
use std::path::{Path, PathBuf};
use std::process;

use reqwest;
use serde::{Deserialize, Serialize};
use serde_yaml;
use url::Url;

/// priority;
/// 1. Command line options
/// 2. URL of the git repository in cwd
///
/// Output error if host is not github.com
pub fn resolve_repository_url(git: &str, remote: &str, opt_repo_url: &str) -> Url {
    let repo_url: Url;
    if opt_repo_url == "" {
        repo_url = git::get_repo_url(git, remote);
    } else {
        repo_url = Url::parse(opt_repo_url).unwrap();
    }
    if repo_url.host_str().unwrap() != "github.com" {
        eprintln!("The repository url is not `github.com`.");
        process::exit(1);
    }

    repo_url
}

pub struct CommitUser {
    pub name: String,
    pub email: String,
}

/// priority;
/// 1. Command line options
/// 2. name and email of the git repository in cwd
pub fn resolve_commit_user(git: &str, opt_name: &str, opt_email: &str) -> CommitUser {
    let mut commit_user = CommitUser {
        name: opt_name.to_string(),
        email: opt_email.to_string(),
    };
    if opt_name == "" {
        commit_user.name = git::get_user_name(git);
    }
    if opt_email == "" {
        commit_user.email = git::get_user_email(git);
    }

    if commit_user.name == "" || commit_user.email == "" {
        eprintln!("Please set the name and email of the user to commit to.");
        process::exit(1);
    }

    commit_user
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

#[derive(Debug)]
enum PathType {
    LocalFile(PathBuf),
    HttpUrl(Url),
}

fn determine_path_type(path: &str) -> Option<PathType> {
    match Url::parse(path) {
        Ok(url) => {
            if url.scheme() == "http" || url.scheme() == "https" {
                Some(PathType::HttpUrl(url))
            } else {
                None
            }
        }
        Err(_) => Some(PathType::LocalFile(Path::new(path).to_path_buf())),
    }
}

fn get_remote_url_content<T: AsRef<str>>(url: T) -> String {
    reqwest::blocking::get(url.as_ref())
        .unwrap()
        .text()
        .unwrap()
}

pub fn load_config(config_file: &str) -> Config {
    let config_file_path = determine_path_type(config_file).unwrap();
    let config_content = match config_file_path {
        PathType::LocalFile(file) => {
            let absolute_file = file.canonicalize().unwrap();
            fs::read_to_string(absolute_file).unwrap()
        }
        PathType::HttpUrl(url) => get_remote_url_content(url),
    };

    let config: Config = serde_yaml::from_str(&config_content).unwrap();
    // TODO validate config

    config
}

pub fn repo_owner(repo_url: &Url) -> String {
    repo_url
        .path_segments()
        .map(|c| c.collect::<Vec<_>>())
        .unwrap()[0]
        .to_string()
}

pub fn repo_name(repo_url: &Url) -> String {
    repo_url
        .path_segments()
        .map(|c| c.collect::<Vec<_>>())
        .unwrap()[1]
        .to_string()
        .replace(".git", "")
}
