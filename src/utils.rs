use crate::git;
use crate::github;
use crate::Scheme;

use anyhow::{anyhow, ensure};
use anyhow::{Context, Result};
use git::RepoUrl;
use reqwest;
use serde::{Deserialize, Serialize};
use serde_yaml;
use std::collections::HashSet;
use std::ffi::OsStr;
use std::fs::File;
use std::hash::Hash;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;
use url::Url;

/// Priority:
///
/// 1. Command line options
/// 2. URL of the git repository in cwd
///
/// Output error if host is not github.com
pub fn resolve_repository_url<S: AsRef<str>>(
    git: impl AsRef<OsStr>,
    cwd: impl AsRef<Path>,
    remote: impl AsRef<str>,
    opt_repo_url: Option<S>,
    opt_scheme: &Scheme,
) -> Result<RepoUrl> {
    Ok(match opt_repo_url {
        Some(opt_repo_url) => RepoUrl::new(opt_repo_url, opt_scheme)?,
        None => git::get_repo_url(&git, &cwd, remote, opt_scheme)?,
    })
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
pub fn resolve_commit_user<S: AsRef<str>>(
    git: impl AsRef<OsStr>,
    cwd: impl AsRef<Path>,
    opt_name: Option<S>,
    opt_email: Option<S>,
) -> Result<CommitUser> {
    let commit_user = CommitUser {
        name: match opt_name {
            Some(opt_name) => opt_name.as_ref().to_string(),
            None => git::get_user_name(&git, &cwd)?,
        },
        email: match opt_email {
            Some(opt_email) => opt_email.as_ref().to_string(),
            None => git::get_user_email(&git, &cwd)?,
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
    pub tools: Vec<Tool>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Tool {
    id: String,
    url: Url,
    language_type: String,
    attachments: Option<Vec<Attachment>>,
    testing: Option<Testing>,
}

impl Tool {
    fn convert_github_url(&self) -> Result<Self> {
        let converted_url =
            github::convert_github_raw_contents_url(&self.url).with_context(|| {
                format!(
                    "Failed to convert the GitHub raw contents URL: {}",
                    &self.url.as_str()
                )
            })?;
        let converted_attachments = match &self.attachments {
            Some(attachments) => Some(
                attachments
                    .iter()
                    .map(|attachment| attachment.convert_github_url())
                    .collect::<Vec<Attachment>>(),
            ),
            None => None,
        };
        let converted_testing = match &self.testing {
            Some(testing) => Some(Testing {
                attachments: testing
                    .attachments
                    .iter()
                    .map(|attachment| attachment.convert_github_url())
                    .collect::<Vec<Attachment>>(),
            }),
            None => None,
        };
        Ok(Self {
            url: converted_url,
            attachments: converted_attachments,
            testing: converted_testing,
            ..self.clone()
        })
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Attachment {
    target: Option<String>,
    url: Url,
}

impl Attachment {
    fn convert_github_url(&self) -> Self {
        let result = github::convert_github_raw_contents_url(&self.url);
        match result {
            Ok(url) => Self {
                url: url,
                ..self.clone()
            },
            Err(_) => Self { ..self.clone() },
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Testing {
    attachments: Vec<Attachment>,
}

/// Load the contents of the file.
/// The config_file can be a local file or a remote file.
pub fn load_config(config_file: impl AsRef<str>) -> Result<String> {
    Ok(match Url::parse(config_file.as_ref()) {
        Ok(url) => {
            // Remote file
            let response = reqwest::blocking::get(url.as_str()).with_context(|| {
                format!("Failed to get from the remote URL: {:?}", url.as_str())
            })?;
            ensure!(
                response.status().is_success(),
                format!("Failed to get from remote URL: {:?}", url)
            );
            response.text().context("Failed to decode response body.")?
        }
        Err(_) => {
            // Local file
            let config_file_path = Path::new(config_file.as_ref())
                .canonicalize()
                .context("Failed to resolve config file path.")?;
            let mut reader = BufReader::new(
                File::open(&config_file_path)
                    .with_context(|| format!("Failed to open file: {:?}", &config_file_path))?,
            );
            let mut content = String::new();
            reader
                .read_to_string(&mut content)
                .with_context(|| format!("Failed to read file: {:?}", config_file_path))?;
            content
        }
    })
}

fn check_duplicate<T>(iter: T) -> bool
where
    T: Iterator,
    T::Item: Eq + Hash,
{
    let mut uniq = HashSet::new();
    iter.into_iter().all(|x| uniq.insert(x))
}

pub fn validate_and_convert_config(config_content: impl AsRef<str>) -> Result<Config> {
    // Validate config_content here by str -> struct
    let config: Config = serde_yaml::from_str(config_content.as_ref())
        .context("Failed to convert to config instance.")?;
    // Check that there are no duplicate id's
    let ids = config
        .tools
        .iter()
        .map(|tool| tool.id.as_str())
        .collect::<Vec<&str>>();
    // let a = check_duplicate(ids);

    // Convert url to github raw-contents url
    Ok(Config {
        tools: config
            .tools
            .iter()
            .map(|tool| tool.convert_github_url())
            .collect::<Result<Vec<Tool>>>()?,
    })
}

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
    use crate::git::{clone, set_commit_user};
    use std::env;
    use std::path::Path;
    use temp_dir::TempDir;
    use url::Url;

    mod resolve_repository_url {
        use super::*;

        #[test]
        fn ok_with_opt() {
            let temp_dir = TempDir::with_prefix("gh-trs").unwrap();
            resolve_repository_url(
                "git",
                temp_dir.path(),
                "origin",
                Some("https://github.com/suecharo/gh-trs.git"),
                &Scheme::Https,
            )
            .unwrap();
        }

        #[test]
        fn ok_with_git_dir() {
            let temp_dir = TempDir::with_prefix("gh-trs").unwrap();
            let repo_url =
                RepoUrl::new("https://github.com/suecharo/gh-trs.git", &Scheme::Https).unwrap();
            clone("git", temp_dir.path(), &repo_url, "main").unwrap();
            resolve_repository_url(
                "git",
                temp_dir.path(),
                "origin",
                None as Option<&str>,
                &Scheme::Https,
            )
            .unwrap();
        }
    }

    mod resolve_commit_user {
        use super::*;

        #[test]
        fn ok_with_opt() {
            let temp_dir = TempDir::with_prefix("gh-trs").unwrap();
            let commit_user = resolve_commit_user(
                "git",
                temp_dir.path(),
                Some("suecharo"),
                Some("foobar@example.com"),
            )
            .unwrap();
            assert_eq!(commit_user.name, "suecharo");
            assert_eq!(commit_user.email, "foobar@example.com");
        }

        #[test]
        fn ok_with_config() {
            let temp_dir = TempDir::with_prefix("gh-trs").unwrap();
            let repo_url =
                RepoUrl::new("https://github.com/suecharo/gh-trs.git", &Scheme::Https).unwrap();
            clone("git", temp_dir.path(), &repo_url, "main").unwrap();
            let commit_user = CommitUser {
                name: "suecharo".to_string(),
                email: "foobar@example.com".to_string(),
            };
            set_commit_user("git", temp_dir.path(), &commit_user).unwrap();

            let result = resolve_commit_user(
                "git",
                temp_dir.path(),
                None as Option<&str>,
                None as Option<&str>,
            )
            .unwrap();
            assert_eq!(result.name, commit_user.name);
            assert_eq!(result.email, commit_user.email);
        }
    }

    mod load_config {
        use super::*;

        #[test]
        fn ok_local_file() {
            let mut cwd = env::current_dir().unwrap();
            cwd.push("tests/gh-trs.test.yml");
            let local_file_path = cwd.canonicalize().unwrap();
            let local_file = Path::new(&local_file_path);
            load_config(local_file.to_str().ok_or("").unwrap()).unwrap();
        }

        #[test]
        fn ok_remote_file() {
            load_config(
                "https://raw.githubusercontent.com/suecharo/gh-trs/main/tests/gh-trs.test.yml",
            )
            .unwrap();
        }

        #[test]
        fn err() {
            assert!(load_config("/tmp/foobar.yml").is_err());
            assert!(load_config(
                "https://raw.githubusercontent.com/suecharo/gh-trs/main/tests/foobar.yml"
            )
            .is_err());
        }
    }

    mod check_duplicate {
        use super::*;

        #[test]
        fn ok() {
            assert!(check_duplicate(vec!(&[] as &[&str])));
            assert!(check_duplicate(vec!(["foo", "foo"])));
        }
    }

    mod validate_and_convert_config {
        use super::*;

        #[test]
        fn ok() {
            let this_file = Path::new(file!()).canonicalize().unwrap();
            let test_config_file = this_file
                .parent()
                .unwrap()
                .parent()
                .unwrap()
                .join("tests/gh-trs.test.yml")
                .canonicalize()
                .unwrap();
            let config_content = load_config(test_config_file.to_str().unwrap()).unwrap();
            validate_and_convert_config(&config_content).unwrap();
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

    mod convert_github_url {
        use super::*;

        #[test]
        fn ok_attachment() {
            let attachment = Attachment {
                target: Some("foobar".to_string()),
                url: Url::parse("https://github.com/suecharo/gh-trs/blob/0fb996810f153be9ad152565227a10e402950953/tests/resources/cwltool/fastqc.cwl").unwrap()
            };
            let converted_attachment = attachment.convert_github_url();
            assert_eq!(converted_attachment.target, Some("foobar".to_string()));
            assert_eq!(converted_attachment.url, Url::parse("https://raw.githubusercontent.com/suecharo/gh-trs/0fb996810f153be9ad152565227a10e402950953/tests/resources/cwltool/fastqc.cwl").unwrap());
        }

        #[test]
        fn ok_attachment_with_other_url() {
            let attachment = Attachment {
                target: Some("foobar".to_string()),
                url: Url::parse("https://example.com").unwrap(),
            };
            let converted_attachment = attachment.convert_github_url();
            assert_eq!(converted_attachment.target, Some("foobar".to_string()));
            assert_eq!(
                converted_attachment.url,
                Url::parse("https://example.com").unwrap()
            );
        }

        #[test]
        fn ok_tool() {
            let tool = Tool {
                id: "foobar".to_string(),
                url: Url::parse("https://github.com/suecharo/gh-trs/blob/0fb996810f153be9ad152565227a10e402950953/tests/resources/cwltool/fastqc.cwl").unwrap(),
                language_type: "foobar".to_string(),
                attachments: None,
                testing: None
            };
            let converted_tool = tool.convert_github_url().unwrap();
            assert_eq!(converted_tool.url, Url::parse("https://raw.githubusercontent.com/suecharo/gh-trs/0fb996810f153be9ad152565227a10e402950953/tests/resources/cwltool/fastqc.cwl").unwrap());
            assert_eq!(converted_tool.language_type, "foobar".to_string());
        }

        #[test]
        fn ok_tool_with_attachments() {
            let attachment = Attachment {
                target: Some("foobar".to_string()),
                url: Url::parse("https://github.com/suecharo/gh-trs/blob/0fb996810f153be9ad152565227a10e402950953/tests/resources/cwltool/fastqc.cwl").unwrap()
            };

            let tool = Tool {
                id: "foobar".to_string(),
                url: Url::parse("https://github.com/suecharo/gh-trs/blob/0fb996810f153be9ad152565227a10e402950953/tests/resources/cwltool/fastqc.cwl").unwrap(),
                language_type: "foobar".to_string(),
                attachments: Some(vec![attachment]),
                testing: None
            };
            let converted_tool = tool.convert_github_url().unwrap();
            assert_eq!(converted_tool.url, Url::parse("https://raw.githubusercontent.com/suecharo/gh-trs/0fb996810f153be9ad152565227a10e402950953/tests/resources/cwltool/fastqc.cwl").unwrap());
            assert_eq!(converted_tool.language_type, "foobar".to_string());
            let converted_attachment = &converted_tool.attachments.ok_or("").unwrap()[0];
            assert_eq!(converted_attachment.target, Some("foobar".to_string()));
            assert_eq!(
                converted_attachment.url,
                Url::parse("https://raw.githubusercontent.com/suecharo/gh-trs/0fb996810f153be9ad152565227a10e402950953/tests/resources/cwltool/fastqc.cwl").unwrap());
        }

        #[test]
        fn err() {
            let tool = Tool {
                id: "foobar".to_string(),
                url: Url::parse("https://example.com").unwrap(),
                language_type: "foobar".to_string(),
                attachments: None,
                testing: None,
            };
            let result = tool.convert_github_url();
            assert!(result.is_err());
        }
    }
}
