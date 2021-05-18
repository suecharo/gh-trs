use anyhow::{anyhow, bail, ensure};
use anyhow::{Context, Result};
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
pub fn convert_github_raw_contents_url(file_url: &str) -> Result<Url> {
    let parsed_url = Url::parse(file_url)
        .with_context(|| format!("Failed to parse the input file URL: {}", file_url))?;
    ensure!(
        ["http", "https", "ftp"].contains(&parsed_url.scheme()),
        "The scheme of the input file URL is not http, https or ftp."
    );
    let host = parsed_url
        .host_str()
        .ok_or(anyhow!("Failed to convert host in parsed URL."))?;
    ensure!(
        host == "github.com" || host == "raw.githubusercontent.com",
        "The input URL host is not github.com or raw.githubusercontent.com"
    );

    let path_segments = parsed_url
        .path_segments()
        .ok_or(anyhow!("Failed to parse path in parsed URL."))?
        .collect::<Vec<&str>>();
    // <repo_owner>/<repo_name>/[blob, hash_value]/<path_to_file>
    ensure!(path_segments.len() >= 4, format!("The path length of the input URL: {:?} is too short. Is it really the GitHub file URL?", file_url));
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
fn is_commit_hash(hash: &str) -> Result<()> {
    let re = Regex::new(r"[0-9a-f]{40}").context("Failed to compile regular expression.")?;
    if re.is_match(hash) {
        Ok(())
    } else {
        bail!(format!("The input string: {} is not a commit hash.", hash))
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

/// Send a GET request to api.github.com to get the latest commit hash
fn get_latest_commit_hash(repo_owner: &str, repo_name: &str, branch: &str) -> Result<String> {
    let url = format!(
        "https://api.github.com/repos/{}/{}/branches/{}",
        repo_owner, repo_name, branch
    );
    let client = reqwest::blocking::Client::new();
    let response = client
        .get(url.as_str())
        .header(reqwest::header::USER_AGENT, "gh-trs")
        .send()
        .with_context(|| format!("Failed to get request to: {:?}", url.as_str()))?;
    ensure!(
        response.status().is_success(),
        format!("Failed to get request to: {:?}", url)
    );
    let body = response
        .json::<ResponseGitHubBranchApi>()
        .context("Failed to json parse response body.")?;
    match is_commit_hash(&body.commit.sha) {
        Ok(_) => Ok(body.commit.sha),
        Err(_) => bail!("Get latest commit hash seems to have failed."),
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
                "https://github.com/suecharo/gh-trs/blob/main/tests/resources/cwltool/fastqc.cwl",
            )
            .unwrap();
            let result_2 = convert_github_raw_contents_url(
                "https://raw.githubusercontent.com/suecharo/gh-trs/main/tests/resources/cwltool/fastqc.cwl",
            )
            .unwrap();
            assert_eq!(result_1, result_2);
        }

        #[test]
        fn ok_hash() {
            let result_1 = convert_github_raw_contents_url(
                "https://github.com/suecharo/gh-trs/blob/0fb996810f153be9ad152565227a10e402950953/tests/resources/cwltool/fastqc.cwl",
            )
            .unwrap();
            let result_2 = convert_github_raw_contents_url(
                "https://raw.githubusercontent.com/suecharo/gh-trs/0fb996810f153be9ad152565227a10e402950953/tests/resources/cwltool/fastqc.cwl",
            )
            .unwrap();
            assert_eq!(result_1, result_2);
        }

        #[test]
        #[should_panic]
        fn local_file_path() {
            convert_github_raw_contents_url("/foo/bar.txt").unwrap();
        }

        #[test]
        #[should_panic]
        fn file_url() {
            convert_github_raw_contents_url("file:///foo/bar.txt").unwrap();
        }

        #[test]
        #[should_panic]
        fn not_github_url() {
            convert_github_raw_contents_url("https://test.com/foo/bar.txt").unwrap();
        }
    }

    mod is_commit_hash {
        use super::*;

        #[test]
        fn ok() {
            is_commit_hash("0fb996810f153be9ad152565227a10e402950953").unwrap();
        }

        #[test]
        #[should_panic]
        fn err() {
            is_commit_hash("foo").unwrap();
            is_commit_hash("main").unwrap();
            is_commit_hash("0fb996810f153be9ad").unwrap();
            is_commit_hash("0fb996810f153be9ad152565227a10e402950953foo").unwrap();
        }
    }

    mod get_latest_commit_hash {
        use super::*;

        #[test]
        fn ok() {
            get_latest_commit_hash("suecharo", "gh-trs", "main").unwrap();
        }

        #[test]
        #[should_panic]
        fn non_existent_branch_name() {
            get_latest_commit_hash("suecharo", "gh-trs", "non_existent_branch").unwrap();
        }
    }
}
