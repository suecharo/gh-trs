use crate::config;
use crate::env;
use crate::github_api;
use crate::inspect;
use crate::raw_url;

use anyhow::{anyhow, Result};
use log::{debug, info};
use std::collections::HashMap;
use std::fs;
use std::io::{BufWriter, Write};
use std::path::Path;
use url::Url;
use uuid::Uuid;

pub fn make_template(
    wf_loc: &Url,
    gh_token: &Option<impl AsRef<str>>,
    output: impl AsRef<Path>,
) -> Result<()> {
    let gh_token = env::github_token(gh_token)?;

    info!(
        "Making a template from workflow location: {}",
        wf_loc.as_str()
    );
    let primary_wf = raw_url::RawUrl::new(
        &gh_token,
        wf_loc,
        &mut None::<HashMap<String, String>>,
        &mut None::<HashMap<String, String>>,
    )?;

    let wf_id = Uuid::new_v4();
    let wf_version = "1.0.0".to_string();
    let wf_license = github_api::get_license(&gh_token, &primary_wf.owner, &primary_wf.name)?;
    let author = github_api::get_author(&gh_token)?;
    let wf_name = primary_wf.file_stem()?;
    let readme = raw_url::RawUrl::new(
        &gh_token,
        &github_api::get_readme_url(&gh_token, &primary_wf.owner, &primary_wf.name)?,
        &mut None::<HashMap<String, String>>,
        &mut None::<HashMap<String, String>>,
    )?
    .to_url()?;
    let language = inspect::inspect_wf_type_version(&primary_wf.to_url()?)?;
    let files = obtain_wf_files(&gh_token, &primary_wf)?;
    let testing = vec![config::Testing::default()];

    let config = config::Config {
        id: wf_id,
        version: wf_version,
        license: wf_license,
        authors: vec![author],
        workflow: config::Workflow {
            name: wf_name,
            readme,
            language,
            files,
            testing,
        },
    };
    debug!("config:\n{:#?}", config);

    let mut buffer = BufWriter::new(fs::File::create(&output)?);
    buffer.write(serde_json::to_string_pretty(&config)?.as_bytes())?;

    Ok(())
}

fn obtain_wf_files(
    gh_token: impl AsRef<str>,
    primary_wf: &raw_url::RawUrl,
) -> Result<Vec<config::File>> {
    let primary_wf_url = primary_wf.to_url()?;
    let base_dir = primary_wf.base_dir()?;
    let base_url = primary_wf.to_base_url()?;
    let files = github_api::get_file_list_recursive(
        gh_token,
        &primary_wf.owner,
        &primary_wf.name,
        &base_dir,
        &primary_wf.commit,
    )?;
    Ok(files
        .into_iter()
        .map(|file| -> Result<config::File> {
            let target = file.strip_prefix(&base_dir)?;
            let url = base_url.join(target.to_str().ok_or(anyhow!("Invalid URL"))?)?;
            let r#type = if url == primary_wf_url {
                config::FileType::Primary
            } else {
                config::FileType::Secondary
            };
            Ok(config::File::new(&url, Some(target), r#type)?)
        })
        .collect::<Result<Vec<_>>>()?)
}