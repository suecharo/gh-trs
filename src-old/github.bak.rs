use crate::git::RepoUrl;
use crate::utils::{repo_name, repo_owner};
use anyhow::{anyhow, bail, ensure, Context, Result};
use regex::Regex;
use reqwest;
use serde::{Deserialize, Serialize};
use url::Url;

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
pub fn convert_github_raw_contents_url(file_url: &Url) -> Result<Url> {
    ensure!(
        ["http", "https"].contains(&file_url.scheme()),
        "The URL scheme of the input file is not http or https."
    );
    let host = file_url
        .host_str()
        .ok_or(anyhow!("Failed to get host_str."))?;
    ensure!(
        host == "github.com" || host == "raw.githubusercontent.com",
        "The entered URL host is not github.com or raw.githubusercontent.com"
    );

    let path_segments = file_url
        .path_segments()
        .ok_or(anyhow!("Failed to parse path in parsed URL."))?
        .collect::<Vec<&str>>();
    // <repo_owner>/<repo_name>/[blob, hash_value]/<path_to_file>
    ensure!(
        path_segments.len() >= 4,
        format!(
            "The path length of input URL: {} is too short. Is it really the URL of a GitHub file?",
            file_url.as_str()
        )
    );
    let commit_hash = if path_segments[2] == "blob" {
        // Ok: suecharo/gh-trs/blob/0fb996810f153be9ad152565227a10e402950953/tests/resources/cwltool/fastqc.cwl
        // Err: suecharo/gh-trs/blob/main/tests/resources/cwltool/fastqc.cwl
        match is_commit_hash(path_segments[3]) {
            Ok(_) => path_segments[3].to_string(),
            Err(_) => get_latest_commit_hash(path_segments[0], path_segments[1], path_segments[3])?,
        }
    } else {
        // Ok: suecharo/gh-trs/0fb996810f153be9ad152565227a10e402950953/tests/resources/cwltool/fastqc.cwl
        // Err: suecharo/gh-trs/main/tests/resources/cwltool/fastqc.cwl
        match is_commit_hash(path_segments[2]) {
            Ok(_) => path_segments[2].to_string(),
            Err(_) => get_latest_commit_hash(path_segments[0], path_segments[1], path_segments[2])?,
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
fn is_commit_hash(hash: impl AsRef<str>) -> Result<()> {
    let re = Regex::new(r"^[0-9a-f]{40}$")?;
    if re.is_match(hash.as_ref()) {
        Ok(())
    } else {
        bail!(format!(
            "The input string: {} is not a commit hash.",
            hash.as_ref()
        ))
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct ResponseGitHubBranchApi {
    commit: ResponseGitHubBranchApiCommit,
}

#[derive(Debug, Deserialize, Serialize)]
struct ResponseGitHubBranchApiCommit {
    sha: String,
}

/// Send a GET request to api.github.com to get the latest commit hash.
///
/// * `repo_owner` - The owner of the repository
/// * `repo_name` - The name of the repository
/// * `branch` - The branch of the repository
fn get_latest_commit_hash(
    repo_owner: impl AsRef<str>,
    repo_name: impl AsRef<str>,
    branch: impl AsRef<str>,
) -> Result<String> {
    let url = format!(
        "https://api.github.com/repos/{}/{}/branches/{}",
        repo_owner.as_ref(),
        repo_name.as_ref(),
        branch.as_ref()
    );
    let client = reqwest::blocking::Client::new();
    let response = client
        .get(&url)
        .header(reqwest::header::USER_AGENT, "gh-trs")
        .send()
        .with_context(|| format!("Failed to get request to: {}", url.as_str()))?;
    ensure!(
        response.status().is_success(),
        format!("Failed to get request to: {}", url.as_str())
    );
    let body = response.json::<ResponseGitHubBranchApi>()?;
    match is_commit_hash(&body.commit.sha) {
        Ok(_) => Ok(body.commit.sha),
        Err(_) => bail!("Failed to get the latest commit hash."),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod convert_github_raw_contents_url {
        use super::*;

        #[test]
        fn ok_branch() {
            let result_1 = convert_github_raw_contents_url(
                &Url::parse("https://github.com/suecharo/gh-trs/blob/main/tests/resources/cwltool/fastqc.cwl").unwrap(),
            )
            .unwrap();
            let result_2 = convert_github_raw_contents_url(
                &Url::parse("https://raw.githubusercontent.com/suecharo/gh-trs/main/tests/resources/cwltool/fastqc.cwl").unwrap(),
            )
            .unwrap();
            assert_eq!(result_1, result_2);
        }

        #[test]
        fn ok_hash() {
            let result_1 = convert_github_raw_contents_url(
                &Url::parse("https://github.com/suecharo/gh-trs/blob/0fb996810f153be9ad152565227a10e402950953/tests/resources/cwltool/fastqc.cwl").unwrap(),
            )
            .unwrap();
            let result_2 = convert_github_raw_contents_url(
                &Url::parse("https://raw.githubusercontent.com/suecharo/gh-trs/0fb996810f153be9ad152565227a10e402950953/tests/resources/cwltool/fastqc.cwl").unwrap(),
            )
            .unwrap();
            assert_eq!(result_1, result_2);
        }

        #[test]
        fn err_file_url() {
            assert!(
                convert_github_raw_contents_url(&Url::parse("file:///foo/bar.txt").unwrap())
                    .is_err()
            );
        }

        #[test]
        fn err_not_github_url() {
            assert!(convert_github_raw_contents_url(
                &Url::parse("https://test.com/foo/bar.txt").unwrap()
            )
            .is_err());
        }
    }

    mod is_commit_hash {
        use super::*;

        #[test]
        fn ok() {
            is_commit_hash("0fb996810f153be9ad152565227a10e402950953").unwrap();
        }

        #[test]
        fn err() {
            assert!(is_commit_hash("foo").is_err());
            assert!(is_commit_hash("main").is_err());
            assert!(is_commit_hash("0fb996810f153be9ad").is_err());
            assert!(is_commit_hash("0fb996810f153be9ad152565227a10e402950953foo").is_err());
        }
    }

    mod get_latest_commit_hash {
        use super::*;

        #[test]
        fn ok() {
            get_latest_commit_hash("suecharo", "gh-trs", "main").unwrap();
        }

        #[test]
        fn err_non_existent_branch_name() {
            assert!(get_latest_commit_hash("suecharo", "gh-trs", "non_existent_branch").is_err());
        }
    }

    mod default_branch_name {
        use super::*;
        use crate::Scheme;

        #[test]
        fn ok() {
            let repo_url =
                RepoUrl::new("https://github.com/suecharo/gh-trs.git", &Scheme::Https).unwrap();
            assert_eq!(default_branch_name(&repo_url).unwrap(), "main");
        }
    }
}
