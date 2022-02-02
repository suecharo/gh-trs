use crate::config;
use crate::env;
use crate::raw_url;

use anyhow::{ensure, Context, Result};
use log::{debug, info};
use serde_yaml;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::BufReader;
use std::path::Path;

#[cfg(not(tarpaulin_include))]
pub fn validate(
    config_file: impl AsRef<Path>,
    gh_token: &Option<impl AsRef<str>>,
) -> Result<config::Config> {
    let gh_token = env::github_token(gh_token)?;

    info!("Validating config file: {}", config_file.as_ref().display());
    let reader = BufReader::new(fs::File::open(config_file)?);
    let config: config::Config = serde_yaml::from_reader(reader)?;
    debug!("config:\n{:#?}", &config);
    let config = validate_and_update_workflow(&gh_token, &config)?;
    debug!("updated config:\n{:#?}", &config);

    Ok(config)
}

fn validate_and_update_workflow(
    gh_token: &impl AsRef<str>,
    config: &config::Config,
) -> Result<config::Config> {
    let mut cloned_config = config.clone();

    let mut branch_memo = HashMap::new();
    let mut commit_memo = HashMap::new();

    cloned_config.workflow.readme = raw_url::RawUrl::new(
        gh_token,
        &config.workflow.readme,
        Some(&mut branch_memo),
        Some(&mut commit_memo),
    )
    .context("Failed to convert readme to raw url")?
    .to_url()?;

    let primary_wf_count = config
        .workflow
        .files
        .iter()
        .filter(|f| f.is_primary())
        .count();
    ensure!(
        primary_wf_count == 1,
        "Expected one primary workflow file. Found {}",
        primary_wf_count
    );

    for i in 0..config.workflow.files.len() {
        cloned_config.workflow.files[i] = config::File::new(
            &raw_url::RawUrl::new(
                gh_token,
                &config.workflow.files[i].url,
                Some(&mut branch_memo),
                Some(&mut commit_memo),
            )
            .with_context(|| {
                format!(
                    "Failed to convert file {} to raw url",
                    &config.workflow.files[i].url
                )
            })?
            .to_url()?,
            config.workflow.files[i].target.clone(),
            config.workflow.files[i].r#type.clone(),
        )?;
    }

    let mut test_id_set: HashSet<&str> = HashSet::new();
    for i in 0..config.workflow.testing.len() {
        ensure!(
            !test_id_set.contains(config.workflow.testing[i].id.as_str()),
            "Duplicate test id: {}",
            config.workflow.testing[i].id
        );
        test_id_set.insert(config.workflow.testing[i].id.as_str());

        for j in 0..config.workflow.testing[i].files.len() {
            match raw_url::RawUrl::new(
                gh_token,
                &config.workflow.testing[i].files[j].url,
                Some(&mut branch_memo),
                Some(&mut commit_memo),
            ) {
                Ok(raw_url) => {
                    cloned_config.workflow.testing[i].files[j] = config::TestFile::new(
                        &raw_url.to_url()?,
                        config.workflow.testing[i].files[j].target.clone(),
                        config.workflow.testing[i].files[j].r#type.clone(),
                    )?
                }
                Err(_) => {}
            };
        }
    }

    Ok(cloned_config)
}
