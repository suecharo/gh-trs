use crate::git;
use crate::github;
use crate::Args;
use anyhow::{anyhow, bail, ensure, Error, Result};
use chrono;
use colour;
use regex::Regex;
use reqwest;
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::env;
use std::fmt;
use std::fs;
use std::fs::File;
use std::hash::Hash;
use std::io::prelude::*;
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};
use url::Url;

pub fn log_info(msg: &str) -> () {
    print!("[{} ", chrono::Local::now().format("%Y-%m-%d %H:%M:%S"));
    colour::green!("INFO");
    println!("] {}", msg);
}

pub fn log_error(msg: Error) -> () {
    eprint!("[{} ", chrono::Local::now().format("%Y-%m-%d %H:%M:%S"));
    colour::red!("ERROR");
    eprintln!("] {}", msg);
}

#[derive(Debug)]
pub struct Context {
    config_file: String,
    pub repo_url: Url,
    pub branch: String,
    pub dest: PathBuf,
    user_name: String,
    pub user_email: String,
    pub github_token: Option<String>,
    default_branch: String,
}

impl Context {
    pub fn new(arg_opt: Args) -> Result<Self> {
        let repo_url = resolve_repository_url(&arg_opt.repo_url)?;
        let user_name = match arg_opt.user_name {
            Some(user_name) => user_name,
            None => git::user_name()?,
        };
        let user_email = match arg_opt.user_email {
            Some(user_email) => user_email,
            None => git::user_email()?,
        };
        ensure!(
            user_name != "" && user_email != "",
            "Please set the name and email of the user to commit to."
        );
        let github_token = match arg_opt.github_token {
            Some(github_token) => Some(github_token),
            None => match env::var("GITHUB_TOKEN") {
                Ok(github_token) => Some(github_token),
                Err(_) => None,
            },
        };
        let default_branch = github::default_branch(&repo_url, &github_token)?;
        Ok(Context {
            config_file: arg_opt.config_file,
            repo_url,
            branch: arg_opt.branch,
            dest: arg_opt.dest,
            user_name,
            user_email,
            github_token,
            default_branch,
        })
    }

    /// Check to be able to request the GitHub REST API.
    /// If the GitHub Token does not exist, do not request.
    pub fn can_api_request(&self) -> Result<bool> {
        match &self.github_token {
            Some(github_token) => {
                let request_limit = github::request_limit(github_token)?;
                Ok(request_limit > 500)
            }
            None => Ok(false),
        }
    }
}

impl fmt::Display for Context {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Config File: {}", self.config_file)?;
        writeln!(f, "Repository URL: {}", self.repo_url)?;
        writeln!(f, "Default Branch: {}", self.default_branch)?;
        writeln!(f, "Pages Branch: {}", self.branch)?;
        writeln!(f, "Pages Destination: {:?}", self.dest)?;
        writeln!(f, "User Name: {}", self.user_name)?;
        writeln!(f, "User Email: {}", self.user_email)?;
        write!(
            f,
            "GitHub Token: {}",
            match &self.github_token {
                Some(_) => "Set",
                None => "Not Set",
            }
        )?;
        Ok(())
    }
}

/// Resolve the repository URL.
/// Output error if host is not github.com
///
/// Priority:
///
/// 1. Command line option: `gh-trs --repo-url`
/// 2. GitHub repository of cwd
///
/// The expected URL to be entered is:
///
/// - https URL like: https://github.com/suecharo/gh-trs.git
/// - ssh URL like: ssh://git@github.com/suecharo/gh-trs.git
/// - ssh (github default) URL like: git@github.com:suecharo/gh-trs.git
///
/// * `opt` - Argument options defined at `main.rs`
fn resolve_repository_url(repo_url: &Option<String>) -> Result<Url> {
    let repo_url = match repo_url {
        Some(repo_url) => repo_url.to_string(),
        None => git::repo_url()?,
    };
    let re_https = Regex::new(r"^https://github\.com/[\w]*/[\w\-]*(\.git)?$")?;
    let re_ssh = Regex::new(r"^ssh://git@github\.com/[\w]*/[\w\-]*(\.git)?$")?;
    let re_ssh_github = Regex::new(r"^git@github\.com:[\w]*/[\w\-]*(\.git)?$")?;
    if re_https.is_match(&repo_url) {
        return Ok(Url::parse(&repo_url)?);
    } else if re_ssh.is_match(&repo_url) {
        return Ok(Url::parse(&repo_url)?);
    } else if re_ssh_github.is_match(&repo_url) {
        return Ok(Url::parse(
            &repo_url.replace("git@github.com:", "ssh://git@github.com/"),
        )?);
    }
    bail!(
        "The inputted URL: {} is not a valid git repository URL.",
        &repo_url
    )
}

/// Extract repository owner from the repository URL
///
/// * `repo_url` - The repository URL.
pub fn repo_owner(repo_url: &Url) -> Result<String> {
    let path_segments = repo_url
        .path_segments()
        .ok_or(anyhow!("Failed to get path segments"))?
        .collect::<Vec<&str>>();
    ensure!(
        path_segments.len() >= 2,
        "The path length of the repository URL is too short."
    );
    Ok(path_segments[0].to_string())
}

/// Extract repository name from the repository URL
///
/// * `repo_url` - The repository URL.
pub fn repo_name(repo_url: &Url) -> Result<String> {
    let path_segments = repo_url
        .path_segments()
        .ok_or(anyhow!("Failed to get path segments"))?
        .collect::<Vec<&str>>();
    ensure!(
        path_segments.len() >= 2,
        "The path length of the repository URL is too short."
    );
    Ok(path_segments[1].to_string())
}

pub fn is_ci_mode() -> Result<bool> {
    unimplemented!();
}

/// Load the contents of the file.
/// The config_file can be a local file or a remote file.
///
/// * `ctx` - The runtime context.
pub fn load_config(ctx: &Context) -> Result<String> {
    Ok(match Url::parse(&ctx.config_file) {
        Ok(url) => {
            // Remote file
            let response = reqwest::blocking::get(url.as_str())?;
            ensure!(
                response.status().is_success(),
                format!("Failed to get from remote URL: {}", url.as_str())
            );
            response.text()?
        }
        Err(_) => {
            // Local file
            let config_file_path = Path::new(&ctx.config_file).canonicalize()?;
            let mut reader = BufReader::new(File::open(&config_file_path)?);
            let mut content = String::new();
            reader.read_to_string(&mut content)?;
            content
        }
    })
}

/// Check duplicate of inputted iterable.
///
/// * `iter` - The iterable to check.
pub fn check_duplicate<T>(iter: T) -> bool
where
    T: IntoIterator,
    T::Item: Eq + Hash,
{
    let mut uniq = HashSet::new();
    iter.into_iter().all(|x| uniq.insert(x))
}

pub fn sha256_digest(content: impl AsRef<[u8]>) -> String {
    let result = Sha256::digest(content.as_ref());
    format!("{:x}", result)
}

// Create dir -> Write file
pub fn dump_file(path: impl AsRef<Path>, content: impl AsRef<[u8]>) -> Result<()> {
    let dir_path = path.as_ref().parent().unwrap();
    fs::create_dir_all(&dir_path)?;
    let mut f = BufWriter::new(fs::File::create(path)?);
    f.write_all(content.as_ref())?;
    Ok(())
}

pub fn load_file(path: impl AsRef<Path>) -> Result<String> {
    let mut reader = BufReader::new(File::open(path)?);
    let mut content = String::new();
    reader.read_to_string(&mut content)?;
    Ok(content)
}
