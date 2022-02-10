use crate::config;
use crate::remote;
use crate::trs_api;

use anyhow::{bail, ensure, Result};
use log::debug;
use serde_json;
use serde_yaml;
use std::fs;
use std::io::BufReader;
use std::io::{BufWriter, Write};
use std::path::Path;
use url::Url;

pub enum FileExt {
    Yaml,
    Json,
}

pub fn parse_file_ext(path: impl AsRef<Path>) -> Result<FileExt> {
    match path.as_ref().extension() {
        Some(ext) => match ext.to_str() {
            Some("yml") => Ok(FileExt::Yaml),
            Some("yaml") => Ok(FileExt::Yaml),
            Some("json") => Ok(FileExt::Json),
            Some(ext) => bail!("Unsupported output file extension: {}", ext),
            None => bail!("Unsupported output file extension"),
        },
        None => Ok(FileExt::Yaml),
    }
}

pub fn write_config(config: &config::Config, path: impl AsRef<Path>, ext: &FileExt) -> Result<()> {
    let content = match ext {
        FileExt::Yaml => serde_yaml::to_string(&config)?,
        FileExt::Json => serde_json::to_string_pretty(&config)?,
    };
    let mut buffer = BufWriter::new(fs::File::create(path)?);
    buffer.write(content.as_bytes())?;

    Ok(())
}

pub fn read_config(location: impl AsRef<str>) -> Result<config::Config> {
    match Url::parse(location.as_ref()) {
        Ok(url) => {
            // as remote url
            // Even json can be read with yaml reader
            let content = remote::fetch_json_content(&url)?;
            Ok(serde_yaml::from_str(&content)?)
        }
        Err(_) => {
            // as local file path
            let reader = BufReader::new(fs::File::open(location.as_ref())?);
            Ok(serde_yaml::from_reader(reader)?)
        }
    }
}

pub fn find_config_loc_recursively_from_trs(trs_loc: impl AsRef<str>) -> Result<Vec<String>> {
    let trs_loc = if trs_loc.as_ref().ends_with("/") {
        trs_loc.as_ref().to_string()
    } else {
        format!("{}/", trs_loc.as_ref())
    };
    let trs_endpoint = trs_api::TrsEndpoint {
        url: Url::parse(&trs_loc)?,
    };
    let service_info = trs_api::get_service_info(&trs_endpoint)?;
    ensure!(
        service_info.r#type.artifact == "gh-trs" && service_info.r#type.version == "2.0.1",
        "gh-trs only supports gh-trs 2.0.1 as a TRS endpoint"
    );
    let config_locs: Vec<String> = trs_api::get_tools(&trs_endpoint)?
        .into_iter()
        .flat_map(|tool| tool.versions)
        .map(|version| version.url)
        .map(|url| format!("{}/gh-trs-config.json", url.as_str()))
        .collect();
    debug!("Found config locations: {:?}", config_locs);
    Ok(config_locs)
}

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod tests {
    use super::*;

    #[test]
    fn test_find_config_loc_recursively_from_trs() -> Result<()> {
        let config_locs =
            find_config_loc_recursively_from_trs("https://suecharo.github.io/gh-trs")?;
        assert!(config_locs.len() > 1);
        Ok(())
    }
}
