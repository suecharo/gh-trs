use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use url::Url;
use uuid::Uuid;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Config {
    pub id: Uuid,
    pub version: String,
    pub license: String,
    pub authors: Vec<String>,
    pub workflow: Workflow,
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
    pub target: PathBuf,
    pub r#type: FileType,
}

impl File {
    pub fn new(url: &Url, target: Option<impl AsRef<Path>>, r#type: FileType) -> Result<Self> {
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
            target: target,
            r#type,
        })
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
                    None::<PathBuf>,
                    TestFileType::WfParams,
                )
                .unwrap(),
                TestFile::new(
                    &Url::parse("https://example.com/path/to/wf_engine_params.json").unwrap(),
                    None::<PathBuf>,
                    TestFileType::WfEngineParams,
                )
                .unwrap(),
                TestFile::new(
                    &Url::parse("https://example.com/path/to/data.fq").unwrap(),
                    None::<PathBuf>,
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
    pub target: PathBuf,
    pub r#type: TestFileType,
}

impl TestFile {
    pub fn new(url: &Url, target: Option<impl AsRef<Path>>, r#type: TestFileType) -> Result<Self> {
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
            target: target,
            r#type,
        })
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TestFileType {
    WfParams,
    WfEngineParams,
    Other,
}
