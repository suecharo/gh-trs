use crate::github_api;
use crate::raw_url;

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use url::Url;
use uuid::Uuid;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Config {
    pub id: Uuid,
    pub version: String,
    pub license: String,
    pub authors: Vec<Author>,
    pub workflow: Workflow,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Author {
    pub github_account: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub affiliation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub orcid: Option<String>,
}

impl Author {
    pub fn new_from_api(gh_token: impl AsRef<str>) -> Result<Self> {
        let (github_account, name, affiliation) = github_api::get_author_info(gh_token)?;
        Ok(Self {
            github_account,
            name: Some(name),
            affiliation: Some(affiliation),
            orcid: None::<String>,
        })
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub name: String,
    pub readme: Url,
    pub language: Language,
    pub files: Vec<File>,
    pub testing: Vec<Testing>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Language {
    pub r#type: Option<LanguageType>,
    pub version: Option<String>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum LanguageType {
    Cwl,
    Wdl,
    Nfl,
    Smk,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct File {
    pub url: Url,
    pub target: Option<PathBuf>,
    pub r#type: FileType,
}

impl File {
    pub fn new(url: &Url, target: &Option<impl AsRef<Path>>, r#type: FileType) -> Result<Self> {
        let target = match target {
            Some(target) => target.as_ref().to_path_buf(),
            None => url
                .path_segments()
                .ok_or(anyhow!("Invalid URL: {}", url.as_ref()))?
                .last()
                .ok_or(anyhow!("Invalid URL: {}", url.as_ref()))?
                .to_string()
                .into(),
        };
        Ok(Self {
            url: url.clone(),
            target: Some(target),
            r#type,
        })
    }

    pub fn is_primary(&self) -> bool {
        self.r#type == FileType::Primary
    }

    /// If the URL's host is github.com and raw.github.com, update the URL to raw_url.
    /// If it isn't these hosts, raise an error.
    pub fn update_url(
        &mut self,
        gh_token: impl AsRef<str>,
        branch_memo: Option<&mut HashMap<String, String>>,
        commit_memo: Option<&mut HashMap<String, String>>,
    ) -> Result<()> {
        self.url = raw_url::RawUrl::new(gh_token, &self.url, branch_memo, commit_memo)
            .with_context(|| format!("Failed to update URL: {} to raw URL", self.url.as_ref()))?
            .to_url()?;
        Ok(())
    }

    pub fn complement_target(&mut self) -> Result<()> {
        if self.target.is_none() {
            let target = self
                .url
                .path_segments()
                .ok_or(anyhow!("Invalid URL: {}", self.url.as_ref()))?
                .last()
                .ok_or(anyhow!("Invalid URL: {}", self.url.as_ref()))?
                .to_string()
                .into();
            self.target = Some(target);
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FileType {
    Primary,
    Secondary,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Testing {
    pub id: String,
    pub files: Vec<TestFile>,
}

impl Default for Testing {
    fn default() -> Self {
        Self {
            id: "test_1".to_string(),
            files: vec![
                TestFile::new(
                    &Url::parse("https://example.com/path/to/wf_params.json").unwrap(),
                    &None::<PathBuf>,
                    TestFileType::WfParams,
                )
                .unwrap(),
                TestFile::new(
                    &Url::parse("https://example.com/path/to/wf_engine_params.json").unwrap(),
                    &None::<PathBuf>,
                    TestFileType::WfEngineParams,
                )
                .unwrap(),
                TestFile::new(
                    &Url::parse("https://example.com/path/to/data.fq").unwrap(),
                    &None::<PathBuf>,
                    TestFileType::Other,
                )
                .unwrap(),
            ],
        }
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct TestFile {
    pub url: Url,
    pub target: Option<PathBuf>,
    pub r#type: TestFileType,
}

impl TestFile {
    pub fn new(url: &Url, target: &Option<impl AsRef<Path>>, r#type: TestFileType) -> Result<Self> {
        let target = match target {
            Some(target) => target.as_ref().to_path_buf(),
            None => url
                .path_segments()
                .ok_or(anyhow!("Invalid URL: {}", url.as_ref()))?
                .last()
                .ok_or(anyhow!("Invalid URL: {}", url.as_ref()))?
                .to_string()
                .into(),
        };
        Ok(Self {
            url: url.clone(),
            target: Some(target),
            r#type,
        })
    }

    /// If the URL's host is github.com and raw.github.com, update the URL to raw_url.
    /// If it isn't these hosts, do nothing.
    pub fn update_url(
        &mut self,
        gh_token: impl AsRef<str>,
        branch_memo: Option<&mut HashMap<String, String>>,
        commit_memo: Option<&mut HashMap<String, String>>,
    ) -> Result<()> {
        match raw_url::RawUrl::new(gh_token, &self.url, branch_memo, commit_memo) {
            Ok(raw_url) => {
                self.url = raw_url.to_url()?;
            }
            Err(_) => {
                // do nothing.
            }
        };
        Ok(())
    }

    pub fn complement_target(&mut self) -> Result<()> {
        if self.target.is_none() {
            let target = self
                .url
                .path_segments()
                .ok_or(anyhow!("Invalid URL: {}", self.url.as_ref()))?
                .last()
                .ok_or(anyhow!("Invalid URL: {}", self.url.as_ref()))?
                .to_string()
                .into();
            self.target = Some(target);
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TestFileType {
    WfParams,
    WfEngineParams,
    Other,
}

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod tests {
    use super::*;

    #[test]
    fn test_file_new() -> Result<()> {
        let url = Url::parse("https://example.com/path/to/file.txt")?;
        let target = Some(PathBuf::from("path/to/file.txt"));
        let file = File::new(&url, &target, FileType::Primary)?;
        assert_eq!(file.url, url);
        assert_eq!(file.target, target);
        assert_eq!(file.r#type, FileType::Primary);
        Ok(())
    }

    #[test]
    fn test_file_new_no_target() -> Result<()> {
        let url = Url::parse("https://example.com/path/to/file.txt")?;
        let file = File::new(&url, &None::<PathBuf>, FileType::Primary)?;
        assert_eq!(file.url, url);
        assert_eq!(file.target, Some(PathBuf::from("file.txt")));
        assert_eq!(file.r#type, FileType::Primary);
        Ok(())
    }

    #[test]
    fn test_testing_default() -> Result<()> {
        let testing = Testing::default();
        assert_eq!(testing.id, "test_1");
        assert_eq!(testing.files.len(), 3);
        Ok(())
    }

    #[test]
    fn test_test_file_new() -> Result<()> {
        let url = Url::parse("https://example.com/path/to/file.txt")?;
        let target = Some(PathBuf::from("path/to/file.txt"));
        let file = TestFile::new(&url, &target, TestFileType::WfParams)?;
        assert_eq!(file.url, url);
        assert_eq!(file.target, target);
        assert_eq!(file.r#type, TestFileType::WfParams);
        Ok(())
    }

    #[test]
    fn test_test_file_no_target() -> Result<()> {
        let url = Url::parse("https://example.com/path/to/file.txt")?;
        let file = TestFile::new(&url, &None::<PathBuf>, TestFileType::WfParams)?;
        assert_eq!(file.url, url);
        assert_eq!(file.target, Some(PathBuf::from("file.txt")));
        assert_eq!(file.r#type, TestFileType::WfParams);
        Ok(())
    }
}
