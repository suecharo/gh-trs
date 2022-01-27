use crate::config::{Language, LanguageType};
use crate::remote;
use anyhow::{anyhow, Result};
use colored::Colorize;
use log::warn;
use regex::Regex;
use serde_yaml;
use std::collections::BTreeMap;
use url::Url;

pub fn inspect_wf_type_version(wf_loc: &Url) -> Result<Language> {
    let wf_content = remote::fetch_raw_content(wf_loc)?;
    let wf_type = inspect_wf_type(&wf_content);
    let wf_version = inspect_wf_version(&wf_content, &wf_type);
    Ok(Language {
        r#type: wf_type,
        version: wf_version,
    })
}

fn inspect_wf_type(wf_content: impl AsRef<str>) -> Option<LanguageType> {
    match check_by_shebang(&wf_content) {
        Some(lang) => Some(lang),
        None => match check_by_regexp(&wf_content) {
            Ok(lang) => lang,
            Err(e) => {
                warn!("{}: {}", "Warning".yellow(), e);
                None
            }
        },
    }
}

fn check_by_shebang(wf_content: impl AsRef<str>) -> Option<LanguageType> {
    let first_line = wf_content.as_ref().lines().next().unwrap_or("");
    if first_line.starts_with("#!") {
        if first_line.contains("cwl") {
            return Some(LanguageType::Cwl);
        } else if first_line.contains("cromwell") {
            return Some(LanguageType::Wdl);
        } else if first_line.contains("nextflow") {
            return Some(LanguageType::Nfl);
        } else if first_line.contains("snakemake") {
            return Some(LanguageType::Smk);
        }
    }
    None
}

fn check_by_regexp(wf_content: impl AsRef<str>) -> Result<Option<LanguageType>> {
    let pattern_wdl = Regex::new(r"^(workflow|task) \w* \{$")?;
    let pattern_nfl = Regex::new(r"^process \w* \{$")?;
    let pattern_smk = Regex::new(r"^rule \w*:$")?;
    for line in wf_content.as_ref().lines() {
        if line.contains("cwlVersion") {
            return Ok(Some(LanguageType::Cwl));
        } else if pattern_wdl.is_match(line) {
            return Ok(Some(LanguageType::Wdl));
        } else if pattern_nfl.is_match(line) {
            return Ok(Some(LanguageType::Nfl));
        } else if pattern_smk.is_match(line) {
            return Ok(Some(LanguageType::Smk));
        }
    }
    Ok(None)
}

pub fn inspect_wf_version(
    wf_content: impl AsRef<str>,
    wf_type: &Option<LanguageType>,
) -> Option<String> {
    match wf_type {
        Some(LanguageType::Cwl) => match inspect_cwl_version(wf_content) {
            Ok(version) => Some(version),
            Err(e) => {
                warn!("{}: {}", "Warning".yellow(), e);
                Some("v1.0".to_string())
            }
        },
        Some(LanguageType::Wdl) => match inspect_wdl_version(wf_content) {
            Ok(version) => Some(version),
            Err(e) => {
                warn!("{}: {}", "Warning".yellow(), e);
                Some("1.0".to_string())
            }
        },
        Some(LanguageType::Nfl) => match inspect_nfl_version(wf_content) {
            Ok(version) => Some(version),
            Err(e) => {
                warn!("{}: {}", "Warning".yellow(), e);
                Some("1.0".to_string())
            }
        },
        Some(LanguageType::Smk) => match inspect_smk_version(wf_content) {
            Ok(version) => Some(version),
            Err(e) => {
                warn!("{}: {}", "Warning".yellow(), e);
                Some("1.0".to_string())
            }
        },
        None => None,
    }
}

/// https://www.commonwl.org/v1.2/CommandLineTool.html#CWLVersion
fn inspect_cwl_version(wf_content: impl AsRef<str>) -> Result<String> {
    let cwl_docs: BTreeMap<String, serde_yaml::Value> = serde_yaml::from_str(wf_content.as_ref())?;
    match cwl_docs.contains_key("cwlVersion") {
        true => match cwl_docs
            .get("cwlVersion")
            .ok_or(anyhow!("Failed to parse cwlVersion"))?
        {
            serde_yaml::Value::String(version) => Ok(version.to_string()),
            _ => Ok("v1.0".to_string()),
        },
        false => Ok("v1.0".to_string()),
    }
}

fn inspect_wdl_version(wf_content: impl AsRef<str>) -> Result<String> {
    let pattern_wdl_version = Regex::new(r"^version \d\.\d$")?;
    for line in wf_content.as_ref().lines() {
        if pattern_wdl_version.is_match(line) {
            let version = line.split_whitespace().nth(1).unwrap_or("1.0");
            return Ok(version.to_string());
        }
    }
    Ok("1.0".to_string())
}

fn inspect_nfl_version(wf_content: impl AsRef<str>) -> Result<String> {
    for line in wf_content.as_ref().lines() {
        if line == "nextflow.enable.dsl=2" {
            return Ok("DSL2".to_string());
        }
    }
    Ok("1.0".to_string())
}

fn inspect_smk_version(_wf_content: impl AsRef<str>) -> Result<String> {
    Ok("1.0".to_string())
}
