use crate::config;
use crate::config_io;
use crate::env;
use crate::raw_url;

use anyhow::{ensure, Context, Result};
use log::{debug, info};
use serde_yaml;
use std::collections::{HashMap, HashSet};

#[cfg(not(tarpaulin_include))]
pub fn validate(
    config_locs: Vec<impl AsRef<str>>,
    gh_token: &Option<impl AsRef<str>>,
) -> Result<Vec<config::Config>> {
    let gh_token = env::github_token(gh_token)?;

    let mut configs = Vec::new();

    for config_loc in config_locs {
        info!("Validating config file: {}", config_loc.as_ref());
        let mut config = config_io::read_config(config_loc.as_ref())?;
        debug!("config:\n{:#?}", &config);

        validate_authors(&config.authors)?;
        validate_language(&config.workflow.language)?;
        validate_wf_name(&config.workflow.name)?;
        validate_and_update_workflow(&gh_token, &mut config)?;

        debug!("Updated config:\n{}", serde_yaml::to_string(&config)?);
        configs.push(config);
    }

    Ok(configs)
}

fn validate_authors(authors: &Vec<config::Author>) -> Result<()> {
    ensure!(authors.len() > 0, "No authors found in config file");
    ensure!(
        authors.len()
            == authors
                .iter()
                .map(|a| a.github_account.clone())
                .collect::<HashSet<_>>()
                .len(),
        "Duplicate github accounts found in config file"
    );
    Ok(())
}

fn validate_language(language: &config::Language) -> Result<()> {
    ensure!(
        language.r#type.is_some(),
        "Language type not specified in config file"
    );
    ensure!(
        language.version.is_some(),
        "Language version not specified in config file"
    );
    Ok(())
}

fn validate_wf_name(wf_name: impl AsRef<str>) -> Result<()> {
    ensure!(
        wf_name
            .as_ref()
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == ' '),
        "Workflow name must be alphanumeric or contain _, -, or space: {}",
        wf_name.as_ref()
    );
    Ok(())
}

fn validate_and_update_workflow(
    gh_token: &impl AsRef<str>,
    config: &mut config::Config,
) -> Result<()> {
    let mut branch_memo = HashMap::new();
    let mut commit_memo = HashMap::new();

    config.workflow.readme = raw_url::RawUrl::new(
        gh_token,
        &config.workflow.readme,
        Some(&mut branch_memo),
        Some(&mut commit_memo),
    )
    .context("Failed to convert readme to raw url")?
    .to_url()?;

    ensure!(
        config.workflow.primary_wf().is_ok(),
        "Expected one primary workflow file."
    );

    for file in &mut config.workflow.files {
        file.update_url(gh_token, Some(&mut branch_memo), Some(&mut commit_memo))?;
        file.complement_target()?;
    }

    let mut test_id_set: HashSet<&str> = HashSet::new();
    for testing in &mut config.workflow.testing {
        ensure!(
            !test_id_set.contains(testing.id.as_str()),
            "Duplicate test id: {}",
            testing.id.as_str()
        );
        test_id_set.insert(testing.id.as_str());

        for file in &mut testing.files {
            file.update_url(gh_token, Some(&mut branch_memo), Some(&mut commit_memo))?;
            file.complement_target()?;
        }
    }

    Ok(())
}
