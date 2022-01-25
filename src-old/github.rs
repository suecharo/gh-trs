use crate::utils;
use anyhow::{anyhow, bail, ensure, Result};
use regex::Regex;
use reqwest;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use url::Url;

#[derive(Debug, Deserialize, Serialize)]
struct ResponseGitHubRepository {
    default_branch: String,
}

/// Send a GET request to api.github.com to get the default branch name of a GitHub repository.
///
/// * `repo_url` - The URL of the repository
/// * `github_token` - The GitHub token to use for authentication
pub fn default_branch(repo_url: &Url, github_token: &Option<String>) -> Result<String> {
    let repo_owner = utils::repo_owner(repo_url)?;
    let repo_name = utils::repo_name(repo_url)?;
    let url = format!(
        "https://api.github.com/repos/{}/{}",
        &repo_owner, &repo_name
    );
    let client = reqwest::blocking::Client::new();
    let request_builder = match github_token {
        Some(github_token) => client
            .get(&url)
            .header(reqwest::header::USER_AGENT, "gh-trs")
            .header(
                reqwest::header::AUTHORIZATION,
                format!("token {}", github_token),
            ),
        None => client
            .get(&url)
            .header(reqwest::header::USER_AGENT, "gh-trs"),
    };
    let response = request_builder.send()?;
    ensure!(
        response.status().is_success(),
        format!("Failed to get request to: {}", &url)
    );
    let body = response.json::<ResponseGitHubRepository>()?;
    Ok(body.default_branch)
}

#[derive(Debug, Deserialize, Serialize)]
struct ResponseRateLimit {
    resources: RateLimitResources,
}

#[derive(Debug, Deserialize, Serialize)]
struct RateLimitResources {
    core: RateLimitCore,
}

#[derive(Debug, Deserialize, Serialize)]
struct RateLimitCore {
    limit: usize,
}

/// Send a GET request to api.github.com to get the rate limit of GitHub REST API.
///
/// * `github_token` - The GitHub token to use for authentication
pub fn request_limit(github_token: &str) -> Result<usize> {
    let client = reqwest::blocking::Client::new();
    let response = client
        .get("https://api.github.com/rate_limit")
        .header(reqwest::header::USER_AGENT, "gh-trs")
        .header(
            reqwest::header::AUTHORIZATION,
            format!("token {}", github_token),
        )
        .send()?;
    ensure!(
        response.status().is_success(),
        "Failed to get GitHub REST API rate limits."
    );
    let body = response.json::<ResponseRateLimit>()?;
    Ok(body.resources.core.limit)
}

pub struct CommitHashCache {
    cache: HashMap<String, String>,
}

/// A cache for commit hashes.
impl CommitHashCache {
    /// Create a new cache.
    pub fn new() -> Self {
        CommitHashCache {
            cache: HashMap::new(),
        }
    }

    /// Get a commit hash from the cache.
    ///
    /// * `repo_owner` - The owner of the repository
    /// * `repo_name` - The name of the repository
    /// * `branch` - The branch name
    /// * `github_token` - The GitHub token to use for authentication
    pub fn get(
        &mut self,
        repo_owner: &str,
        repo_name: &str,
        branch: &str,
        github_token: &str,
    ) -> Result<String> {
        let cache_key = format!("{}/{}/{}", repo_owner, repo_name, branch);
        match self.cache.get(&cache_key) {
            Some(value) => Ok(value.to_string()),
            None => {
                let latest_commit_hash =
                    get_latest_commit_hash(repo_owner, repo_name, branch, github_token)?;
                self.cache.insert(cache_key, latest_commit_hash.clone());
                Ok(latest_commit_hash)
            }
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct ResponseBranchApi {
    commit: BranchApiCommit,
}

#[derive(Debug, Deserialize, Serialize)]
struct BranchApiCommit {
    sha: String,
}

/// Send a GET request to api.github.com to get the latest commit hash.
///
/// * `repo_owner` - The owner of the repository
/// * `repo_name` - The name of the repository
/// * `branch` - The branch of the repository
/// * `github_token` - The GitHub token to use for authentication
fn get_latest_commit_hash(
    repo_owner: &str,
    repo_name: &str,
    branch: &str,
    github_token: &str,
) -> Result<String> {
    let url = format!(
        "https://api.github.com/repos/{}/{}/branches/{}",
        repo_owner, repo_name, branch
    );
    let client = reqwest::blocking::Client::new();
    let response = client
        .get(&url)
        .header(reqwest::header::USER_AGENT, "gh-trs")
        .header(
            reqwest::header::AUTHORIZATION,
            format!("token {}", github_token),
        )
        .send()?;
    ensure!(
        response.status().is_success(),
        format!("Failed to get request to: {}", &url)
    );
    let body = response.json::<ResponseBranchApi>()?;
    match is_commit_hash(&body.commit.sha) {
        Ok(_) => Ok(body.commit.sha),
        Err(_) => bail!("Failed to get the latest commit hash."),
    }
}

/// Convert the input string into a unique github raw contents url
///
/// Possible inputs:
///
/// - `https://github.com/suecharo/gh-trs/blob/main/tests/resources/cwltool/fastqc.cwl`
/// - `https://raw.githubusercontent.com/suecharo/gh-trs/main/tests/resources/cwltool/fastqc.cwl`
/// - `https://github.com/suecharo/gh-trs/blob/0fb996810f153be9ad152565227a10e402950953/tests/resources/cwltool/fastqc.cwl`
/// - `https://raw.githubusercontent.com/suecharo/gh-trs/0fb996810f153be9ad152565227a10e402950953/tests/resources/cwltool/fastqc.cwl`
///
/// Expected output:
///
/// - `https://raw.githubusercontent.com/suecharo/gh-trs/0fb996810f153be9ad152565227a10e402950953/tests/resources/cwltool/fastqc.cwl`
///
/// If the url is not github.com or raw.githubusercontent.com, return an Error.
///
/// * `file_url` - The file url to convert
/// * `cache` - The cache for commit hashes
/// * `github_token` - The GitHub token to use for authentication
pub fn convert_github_raw_contents_url(
    file_url: &Url,
    cache: &mut CommitHashCache,
    github_token: &str,
) -> Result<Url> {
    ensure!(
        ["http", "https"].contains(&file_url.scheme()),
        "The URL scheme of the input URL is not http or https."
    );
    match file_url.host_str() {
        Some(host) => {
            ensure!(
                host == "github.com" || host == "raw.githubusercontent.com",
                "The host of the input URL is not github.com or raw.githubusercontent.com"
            );
        }
        None => {
            bail!("Failed to get host_str.");
        }
    }
    // <repo_owner>/<repo_name>/[blob, hash_value]/<path_to_file>
    let path_segments = file_url
        .path_segments()
        .ok_or(anyhow!("Failed to parse path in parsed URL."))?
        .collect::<Vec<&str>>();
    ensure!(
        path_segments.len() >= 4,
        "The path length of input URL is too short"
    );
    let commit_hash = if path_segments[2] == "blob" {
        // Ok: suecharo/gh-trs/blob/0fb996810f153be9ad152565227a10e402950953/tests/resources/cwltool/fastqc.cwl
        // Err: suecharo/gh-trs/blob/main/tests/resources/cwltool/fastqc.cwl
        match is_commit_hash(path_segments[3]) {
            Ok(_) => path_segments[3].to_string(),
            Err(_) => cache.get(
                path_segments[0],
                path_segments[1],
                path_segments[3],
                github_token,
            )?,
        }
    } else {
        // Ok: suecharo/gh-trs/0fb996810f153be9ad152565227a10e402950953/tests/resources/cwltool/fastqc.cwl
        // Err: suecharo/gh-trs/main/tests/resources/cwltool/fastqc.cwl
        match is_commit_hash(path_segments[2]) {
            Ok(_) => path_segments[2].to_string(),
            Err(_) => cache.get(
                path_segments[0],
                path_segments[1],
                path_segments[2],
                github_token,
            )?,
        }
    };
    let file_path = if path_segments[2] == "blob" {
        path_segments[4..].join("/")
    } else {
        path_segments[3..].join("/")
    };
    Ok(Url::parse(&format!(
        "https://raw.githubusercontent.com/{repo_owner}/{repo_name}/{commit_hash}/{file_path}",
        repo_owner = path_segments[0],
        repo_name = path_segments[1],
        commit_hash = commit_hash,
        file_path = file_path
    ))?)
}

/// Check if a str is in a 40 character git commit hash.
///
/// * `str` - The string to check
fn is_commit_hash(hash: &str) -> Result<()> {
    let re = Regex::new(r"^[0-9a-f]{40}$")?;
    if re.is_match(hash.as_ref()) {
        Ok(())
    } else {
        bail!(format!("The input string: {} is not a commit hash.", hash))
    }
}
