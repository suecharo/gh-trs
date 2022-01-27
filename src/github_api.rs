use anyhow::{anyhow, bail, ensure, Result};
use reqwest;
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use url::Url;

fn get_request(gh_token: impl AsRef<str>, url: &Url, query: &[(&str, &str)]) -> Result<Value> {
    let client = reqwest::blocking::Client::new();
    let response = client
        .get(url.as_str())
        .header(reqwest::header::USER_AGENT, "gh-trs")
        .header(reqwest::header::ACCEPT, "application/vnd.github.v3+json")
        .header(
            reqwest::header::AUTHORIZATION,
            format!("token {}", gh_token.as_ref()),
        )
        .query(query)
        .send()?;
    let status = response.status();
    let res_body = response.json::<Value>()?;
    ensure!(
        status != reqwest::StatusCode::UNAUTHORIZED,
        "Failed to authenticate with GitHub. Please check your GitHub token."
    );
    ensure!(
        status.is_success(),
        "Failed to get request to {}. Response: {}",
        url,
        match res_body.get("message") {
            Some(message) => message.as_str().unwrap_or_else(|| status.as_str()),
            None => status.as_str(),
        }
    );
    Ok(res_body)
}

fn post_request(gh_token: impl AsRef<str>, url: &Url) -> Result<Value> {
    let client = reqwest::blocking::Client::new();
    let response = client
        .post(url.as_str())
        .header(reqwest::header::USER_AGENT, "gh-trs")
        .header(reqwest::header::ACCEPT, "application/vnd.github.v3+json")
        .header(
            reqwest::header::AUTHORIZATION,
            format!("token {}", gh_token.as_ref()),
        )
        .send()?;
    let status = response.status();
    let res_body = response.json::<Value>()?;
    ensure!(
        status != reqwest::StatusCode::UNAUTHORIZED,
        "Failed to authenticate with GitHub. Please check your GitHub token."
    );
    ensure!(
        status.is_success(),
        "Failed to post request to {}. Response: {}",
        url,
        match res_body.get("message") {
            Some(message) => message.as_str().unwrap_or_else(|| status.as_str()),
            None => status.as_str(),
        }
    );
    Ok(res_body)
}

/// https://docs.github.com/ja/rest/reference/repos#get-a-repository
fn get_repos(
    gh_token: impl AsRef<str>,
    owner: impl AsRef<str>,
    name: impl AsRef<str>,
) -> Result<Value> {
    let url = Url::parse(&format!(
        "https://api.github.com/repos/{}/{}",
        owner.as_ref(),
        name.as_ref()
    ))?;
    get_request(gh_token, &url, &[])
}

pub fn get_default_branch(
    gh_token: impl AsRef<str>,
    owner: impl AsRef<str>,
    name: impl AsRef<str>,
    memo: Option<&mut HashMap<String, String>>,
) -> Result<String> {
    let err_message = "Failed to parse the response when getting default branch";
    match memo {
        Some(memo) => {
            let key = format!("{}/{}", owner.as_ref(), name.as_ref());
            match memo.get(&key) {
                Some(default_branch) => Ok(default_branch.to_string()),
                None => {
                    let res = get_repos(gh_token, owner, name)?;
                    let default_branch = res
                        .get("default_branch")
                        .ok_or(anyhow!(err_message))?
                        .as_str()
                        .ok_or(anyhow!(err_message))?
                        .to_string();
                    memo.insert(key, default_branch.clone());
                    Ok(default_branch)
                }
            }
        }
        None => {
            let res = get_repos(gh_token, owner, name)?;
            Ok(res
                .get("default_branch")
                .ok_or(anyhow!(err_message))?
                .as_str()
                .ok_or(anyhow!(err_message))?
                .to_string())
        }
    }
}

pub fn get_license(
    gh_token: impl AsRef<str>,
    owner: impl AsRef<str>,
    name: impl AsRef<str>,
) -> Result<String> {
    let res = get_repos(gh_token, owner, name)?;
    let err_message = "Failed to parse the response when getting license";
    Ok(res
        .get("license")
        .ok_or(anyhow!(err_message))?
        .get("spdx_id")
        .ok_or(anyhow!(err_message))?
        .as_str()
        .ok_or(anyhow!(err_message))?
        .to_string())
}

/// https://docs.github.com/ja/rest/reference/branches#get-a-branch
fn get_branches(
    gh_token: impl AsRef<str>,
    owner: impl AsRef<str>,
    name: impl AsRef<str>,
    branch: impl AsRef<str>,
) -> Result<Value> {
    let url = Url::parse(&format!(
        "https://api.github.com/repos/{}/{}/branches/{}",
        owner.as_ref(),
        name.as_ref(),
        branch.as_ref()
    ))?;
    get_request(gh_token, &url, &[])
}

pub fn get_latest_commit_hash(
    gh_token: impl AsRef<str>,
    owner: impl AsRef<str>,
    name: impl AsRef<str>,
    branch: impl AsRef<str>,
    memo: Option<&mut HashMap<String, String>>,
) -> Result<String> {
    let err_message = "Failed to parse the response when getting latest commit hash";
    match memo {
        Some(memo) => {
            let key = format!("{}/{}/{}", owner.as_ref(), name.as_ref(), branch.as_ref());
            match memo.get(&key) {
                Some(latest_commit_hash) => Ok(latest_commit_hash.to_string()),
                None => {
                    let res = get_branches(gh_token, owner, name, branch)?;
                    let latest_commit_hash = res
                        .get("commit")
                        .ok_or(anyhow!(err_message))?
                        .get("sha")
                        .ok_or(anyhow!(err_message))?
                        .as_str()
                        .ok_or(anyhow!(err_message))?
                        .to_string();
                    memo.insert(key, latest_commit_hash.clone());
                    Ok(latest_commit_hash)
                }
            }
        }
        None => {
            let res = get_branches(gh_token, owner, name, branch)?;
            Ok(res
                .get("commit")
                .ok_or(anyhow!(err_message))?
                .get("sha")
                .ok_or(anyhow!(err_message))?
                .as_str()
                .ok_or(anyhow!(err_message))?
                .to_string())
        }
    }
}

/// https://docs.github.com/ja/rest/reference/users#get-a-user
#[cfg(not(tarpaulin))]
fn get_user(gh_token: impl AsRef<str>) -> Result<Value> {
    let url = Url::parse("https://api.github.com/user")?;
    get_request(gh_token, &url, &[])
}

#[cfg(not(tarpaulin))]
pub fn get_author(gh_token: impl AsRef<str>) -> Result<String> {
    let res = get_user(gh_token)?;
    let err_message = "Failed to parse the response when getting author";
    let gh_account = res
        .get("login")
        .ok_or(anyhow!(err_message))?
        .as_str()
        .ok_or(anyhow!(err_message))?;
    let name = res
        .get("name")
        .ok_or(anyhow!(err_message))?
        .as_str()
        .ok_or(anyhow!(err_message))?;
    Ok(format!("{} <@{}>", name, gh_account))
}

///

/// https://docs.github.com/ja/rest/reference/repos#get-a-repository-readme
pub fn get_readme_url(
    gh_token: impl AsRef<str>,
    owner: impl AsRef<str>,
    name: impl AsRef<str>,
) -> Result<Url> {
    let url = Url::parse(&format!(
        "https://api.github.com/repos/{}/{}/readme",
        owner.as_ref(),
        name.as_ref()
    ))?;
    let res = get_request(gh_token, &url, &[])?;
    let err_message = "Failed to parse the response when getting readme url.";
    Ok(Url::parse(
        res.get("html_url")
            .ok_or(anyhow!(err_message))?
            .as_str()
            .ok_or(anyhow!(err_message))?,
    )?)
}

/// https://docs.github.com/ja/rest/reference/repos#get-repository-content
fn get_contents(
    gh_token: impl AsRef<str>,
    owner: impl AsRef<str>,
    name: impl AsRef<str>,
    path: impl AsRef<Path>,
    commit: impl AsRef<str>,
) -> Result<Value> {
    let url = Url::parse(&format!(
        "https://api.github.com/repos/{}/{}/contents/{}",
        owner.as_ref(),
        name.as_ref(),
        path.as_ref().display()
    ))?;
    get_request(gh_token, &url, &[("ref", commit.as_ref())])
}

pub fn get_file_list_recursive(
    github_token: impl AsRef<str>,
    owner: impl AsRef<str>,
    name: impl AsRef<str>,
    path: impl AsRef<Path>,
    commit: impl AsRef<str>,
) -> Result<Vec<PathBuf>> {
    let res = get_contents(
        github_token.as_ref(),
        owner.as_ref(),
        name.as_ref(),
        path,
        commit.as_ref(),
    )?;
    let err_message = "Failed to parse the response when getting file list.";
    match res.as_array() {
        Some(files) => {
            let mut file_list: Vec<PathBuf> = Vec::new();
            for file in files {
                let path = PathBuf::from(
                    file.get("path")
                        .ok_or(anyhow!(err_message))?
                        .as_str()
                        .ok_or(anyhow!(err_message))?,
                );
                let r#type = file
                    .get("type")
                    .ok_or(anyhow!(err_message))?
                    .as_str()
                    .ok_or(anyhow!(err_message))?;
                match r#type {
                    "file" => file_list.push(path),
                    "dir" => {
                        let mut sub_file_list = get_file_list_recursive(
                            github_token.as_ref(),
                            owner.as_ref(),
                            name.as_ref(),
                            path,
                            commit.as_ref(),
                        )?;
                        file_list.append(&mut sub_file_list);
                    }
                    _ => {}
                }
            }
            Ok(file_list)
        }
        None => bail!(err_message),
    }
}

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod tests {
    use super::*;
    use crate::env;

    #[test]
    fn test_get_default_branch() -> Result<()> {
        let gh_token = env::github_token(&None::<String>)?;
        let branch = get_default_branch(
            &gh_token,
            "suecharo",
            "gh-trs",
            None::<&mut HashMap<String, String>>,
        )?;
        assert_eq!(branch, "main");
        Ok(())
    }

    #[test]
    fn test_get_default_branch_with_memo() -> Result<()> {
        let gh_token = env::github_token(&None::<String>)?;
        let mut memo = HashMap::new();
        get_default_branch(&gh_token, "suecharo", "gh-trs", Some(&mut memo))?;
        get_default_branch(&gh_token, "suecharo", "gh-trs", Some(&mut memo))?;
        Ok(())
    }

    #[test]
    fn test_get_latest_commit_hash() -> Result<()> {
        let gh_token = env::github_token(&None::<String>)?;
        get_latest_commit_hash(
            &gh_token,
            "suecharo",
            "gh-trs",
            "main",
            None::<&mut HashMap<String, String>>,
        )?;
        Ok(())
    }

    #[test]
    fn test_get_latest_commit_hash_with_memo() -> Result<()> {
        let gh_token = env::github_token(&None::<String>)?;
        let mut memo = HashMap::new();
        get_latest_commit_hash(&gh_token, "suecharo", "gh-trs", "main", Some(&mut memo))?;
        get_latest_commit_hash(&gh_token, "suecharo", "gh-trs", "main", Some(&mut memo))?;
        Ok(())
    }

    #[test]
    #[cfg(not(tarpaulin))]
    fn test_get_author() -> Result<()> {
        let gh_token = env::github_token(&None::<String>)?;
        get_author(&gh_token)?;
        Ok(())
    }

    #[test]
    fn test_get_readme_url() -> Result<()> {
        let gh_token = env::github_token(&None::<String>)?;
        let readme_url = get_readme_url(&gh_token, "suecharo", "gh-trs")?;
        assert_eq!(
            readme_url.to_string().as_str(),
            "https://github.com/suecharo/gh-trs/blob/main/README.md"
        );
        Ok(())
    }

    #[test]
    fn test_get_file_list_recursive() -> Result<()> {
        let gh_token = env::github_token(&None::<String>)?;
        let file_list = get_file_list_recursive(&gh_token, "suecharo", "gh-trs", ".", "main")?;
        assert!(file_list.contains(&PathBuf::from("README.md")));
        assert!(file_list.contains(&PathBuf::from("LICENSE")));
        assert!(file_list.contains(&PathBuf::from("src/main.rs")));
        Ok(())
    }

    #[test]
    fn test_get_file_list_recursive_with_dir() -> Result<()> {
        let gh_token = env::github_token(&None::<String>)?;
        let file_list = get_file_list_recursive(&gh_token, "suecharo", "gh-trs", "src", "main")?;
        assert!(file_list.contains(&PathBuf::from("src/main.rs")));
        Ok(())
    }

    #[test]
    fn test_get_license() -> Result<()> {
        let gh_token = env::github_token(&None::<String>)?;
        let license = get_license(&gh_token, "suecharo", "gh-trs")?;
        assert_eq!(license, "Apache-2.0".to_string());
        Ok(())
    }
}
