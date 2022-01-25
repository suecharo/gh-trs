use crate::git;
use crate::github;
use crate::Opt;
use anyhow::{bail, ensure, Context, Result};
use regex::Regex;
use reqwest;
use serde::{Deserialize, Serialize};
use serde_yaml;
use std::collections::HashSet;
use std::fs::File;
use std::hash::Hash;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;
use url::Url;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub tools: Vec<Tool>,
}

impl Config {
    #[allow(dead_code)]
    pub fn new() -> Result<Self> {
        Ok(Config { tools: Vec::new() })
    }

    pub fn extract_tool_ids(&self) -> Result<Vec<&str>> {
        Ok(self
            .tools
            .iter()
            .map(|tool| tool.id.as_str())
            .collect::<Vec<&str>>())
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Tool {
    pub id: String,
    pub url: Url,
    pub language_type: String,
    pub attachments: Option<Vec<Attachment>>,
    pub testing: Option<Testing>,
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

/// Check duplicate of inputted iterable.
///
/// * `iter` - The iterable to check.
fn check_duplicate<T>(iter: T) -> bool
where
    T: IntoIterator,
    T::Item: Eq + Hash,
{
    let mut uniq = HashSet::new();
    iter.into_iter().all(|x| uniq.insert(x))
}

/// Validate the contents of the config file.
///
/// * `config_content` - The config file contents.
pub fn validate_and_convert_config(config_content: impl AsRef<str>) -> Result<Config> {
    // Validate config_content here by str -> struct
    let config: Config = serde_yaml::from_str(config_content.as_ref())?;
    // Check that there are no duplicate id's
    let ids = config.extract_tool_ids()?;
    if !check_duplicate(ids) {
        bail!("There is a duplicate tool id.")
    }

    // Convert url to github raw-contents url
    Ok(Config {
        tools: config
            .tools
            .iter()
            .map(|tool| tool.convert_github_url())
            .collect::<Result<Vec<Tool>>>()?,
    })
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::Scheme;
//     use std::env;
//     use std::path::Path;
//     use structopt::StructOpt;
//     use url::Url;

//     mod resolve_repository_url {
//         use super::*;

//         #[test]
//         fn ok_with_opt() {
//             let opt = Opt::from_iter(&[
//                 "gh-trs",
//                 "gh-trs.yml",
//                 "--repo-url",
//                 "https://github.com/suecharo/gh-trs.git",
//             ]);
//             resolve_repository_url(&opt).unwrap();
//         }

//         #[test]
//         fn ok_with_git_dir() {
//             let opt = Opt::from_iter(&["gh-trs", "gh-trs.yml"]);
//             resolve_repository_url(&opt).unwrap();
//         }
//     }

//     mod load_config {
//         use super::*;

//         #[test]
//         fn ok_local_file() {
//             let mut cwd = env::current_dir().unwrap();
//             cwd.push("tests/gh-trs.test.yml");
//             let local_file_path = cwd.canonicalize().unwrap();
//             let local_file = Path::new(&local_file_path);
//             load_config(local_file.to_str().ok_or("").unwrap()).unwrap();
//         }

//         #[test]
//         fn ok_remote_file() {
//             load_config(
//                 "https://raw.githubusercontent.com/suecharo/gh-trs/main/tests/gh-trs.test.yml",
//             )
//             .unwrap();
//         }

//         #[test]
//         fn err() {
//             assert!(load_config("/tmp/foobar.yml").is_err());
//             assert!(load_config(
//                 "https://raw.githubusercontent.com/suecharo/gh-trs/main/tests/foobar.yml"
//             )
//             .is_err());
//         }
//     }

//     mod check_duplicate {
//         use super::*;

//         #[test]
//         fn ok() {
//             assert!(check_duplicate(&[""]));
//             assert!(check_duplicate(&["foo", "bar"]));
//             assert_eq!(check_duplicate(&["foo", "foo", "bar"]), false);
//         }
//     }

//     mod validate_and_convert_config {
//         use super::*;

//         #[test]
//         fn ok() {
//             let this_file = Path::new(file!()).canonicalize().unwrap();
//             let test_config_file = this_file
//                 .parent()
//                 .unwrap()
//                 .parent()
//                 .unwrap()
//                 .join("tests/gh-trs.test.yml")
//                 .canonicalize()
//                 .unwrap();
//             let config_content = load_config(test_config_file.to_str().unwrap()).unwrap();
//             validate_and_convert_config(&config_content).unwrap();
//         }
//     }

//     mod repo_owner {
//         use super::*;

//         #[test]
//         fn ok() {
//             let repo_url =
//                 RepoUrl::new("https://github.com/suecharo/gh-trs.git", &Scheme::Https).unwrap();
//             assert_eq!(repo_owner(&repo_url).unwrap(), "suecharo");
//         }
//     }

//     mod repo_name {
//         use super::*;

//         #[test]
//         fn ok() {
//             let repo_url =
//                 RepoUrl::new("https://github.com/suecharo/gh-trs.git", &Scheme::Https).unwrap();
//             assert_eq!(repo_name(&repo_url).unwrap(), "gh-trs");
//         }
//     }

//     mod convert_github_url {
//         use super::*;

//         #[test]
//         fn ok_attachment() {
//             let attachment = Attachment {
//                 target: Some("foobar".to_string()),
//                 url: Url::parse("https://github.com/suecharo/gh-trs/blob/0fb996810f153be9ad152565227a10e402950953/tests/resources/cwltool/fastqc.cwl").unwrap()
//             };
//             let converted_attachment = attachment.convert_github_url();
//             assert_eq!(converted_attachment.target, Some("foobar".to_string()));
//             assert_eq!(converted_attachment.url, Url::parse("https://raw.githubusercontent.com/suecharo/gh-trs/0fb996810f153be9ad152565227a10e402950953/tests/resources/cwltool/fastqc.cwl").unwrap());
//         }

//         #[test]
//         fn ok_attachment_with_other_url() {
//             let attachment = Attachment {
//                 target: Some("foobar".to_string()),
//                 url: Url::parse("https://example.com").unwrap(),
//             };
//             let converted_attachment = attachment.convert_github_url();
//             assert_eq!(converted_attachment.target, Some("foobar".to_string()));
//             assert_eq!(
//                 converted_attachment.url,
//                 Url::parse("https://example.com").unwrap()
//             );
//         }

//         #[test]
//         fn ok_tool() {
//             let tool = Tool {
//                 id: "foobar".to_string(),
//                 url: Url::parse("https://github.com/suecharo/gh-trs/blob/0fb996810f153be9ad152565227a10e402950953/tests/resources/cwltool/fastqc.cwl").unwrap(),
//                 language_type: "foobar".to_string(),
//                 attachments: None,
//                 testing: None
//             };
//             let converted_tool = tool.convert_github_url().unwrap();
//             assert_eq!(converted_tool.url, Url::parse("https://raw.githubusercontent.com/suecharo/gh-trs/0fb996810f153be9ad152565227a10e402950953/tests/resources/cwltool/fastqc.cwl").unwrap());
//             assert_eq!(converted_tool.language_type, "foobar".to_string());
//         }

//         #[test]
//         fn ok_tool_with_attachments() {
//             let attachment = Attachment {
//                 target: Some("foobar".to_string()),
//                 url: Url::parse("https://github.com/suecharo/gh-trs/blob/0fb996810f153be9ad152565227a10e402950953/tests/resources/cwltool/fastqc.cwl").unwrap()
//             };

//             let tool = Tool {
//                 id: "foobar".to_string(),
//                 url: Url::parse("https://github.com/suecharo/gh-trs/blob/0fb996810f153be9ad152565227a10e402950953/tests/resources/cwltool/fastqc.cwl").unwrap(),
//                 language_type: "foobar".to_string(),
//                 attachments: Some(vec![attachment]),
//                 testing: None
//             };
//             let converted_tool = tool.convert_github_url().unwrap();
//             assert_eq!(converted_tool.url, Url::parse("https://raw.githubusercontent.com/suecharo/gh-trs/0fb996810f153be9ad152565227a10e402950953/tests/resources/cwltool/fastqc.cwl").unwrap());
//             assert_eq!(converted_tool.language_type, "foobar".to_string());
//             let converted_attachment = &converted_tool.attachments.ok_or("").unwrap()[0];
//             assert_eq!(converted_attachment.target, Some("foobar".to_string()));
//             assert_eq!(
//                 converted_attachment.url,
//                 Url::parse("https://raw.githubusercontent.com/suecharo/gh-trs/0fb996810f153be9ad152565227a10e402950953/tests/resources/cwltool/fastqc.cwl").unwrap());
//         }

//         #[test]
//         fn err() {
//             let tool = Tool {
//                 id: "foobar".to_string(),
//                 url: Url::parse("https://example.com").unwrap(),
//                 language_type: "foobar".to_string(),
//                 attachments: None,
//                 testing: None,
//             };
//             let result = tool.convert_github_url();
//             assert!(result.is_err());
//         }
//     }
// }
