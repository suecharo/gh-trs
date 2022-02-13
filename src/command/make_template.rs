use crate::config;
use crate::env;
use crate::github_api;
use crate::inspect;
use crate::raw_url;

use anyhow::{anyhow, Result};
use log::{debug, info};
use std::path::Path;
use url::Url;
use uuid::Uuid;

#[cfg(not(tarpaulin_include))]
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
    let primary_wf = raw_url::RawUrl::new(&gh_token, wf_loc, None, None)?;

    let wf_id = Uuid::new_v4();
    let wf_version = "1.0.0".to_string();
    let author = config::types::Author {
        github_account: config::types::Author::new_from_api(&gh_token)?.github_account,
        name: None,
        affiliation: None,
        orcid: None,
    };
    let wf_name = primary_wf.file_stem()?;
    let readme = raw_url::RawUrl::new(
        &gh_token,
        &github_api::get_readme_url(&gh_token, &primary_wf.owner, &primary_wf.name)?,
        None,
        None,
    )?
    .to_url()?;
    let language = inspect::inspect_wf_type_version(&primary_wf.to_url()?)?;
    let files = obtain_wf_files(&gh_token, &primary_wf)?;
    let testing = vec![config::types::Testing::default()];

    let config = config::types::Config {
        id: wf_id,
        version: wf_version,
        license: None,
        authors: vec![author],
        workflow: config::types::Workflow {
            name: wf_name,
            readme,
            language,
            files,
            testing,
        },
    };
    debug!("config:\n{:#?}", config);

    let file_ext = config::io::parse_file_ext(&output)?;
    config::io::write_config(&config, &output, &file_ext)?;

    Ok(())
}

fn obtain_wf_files(
    gh_token: impl AsRef<str>,
    primary_wf: &raw_url::RawUrl,
) -> Result<Vec<config::types::File>> {
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
        .map(|file| -> Result<config::types::File> {
            let target = file.strip_prefix(&base_dir)?;
            let url = base_url.join(target.to_str().ok_or(anyhow!("Invalid URL"))?)?;
            let r#type = if url == primary_wf_url {
                config::types::FileType::Primary
            } else {
                config::types::FileType::Secondary
            };
            Ok(config::types::File::new(&url, &Some(target), r#type)?)
        })
        .collect::<Result<Vec<_>>>()?)
}

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod tests {
    use super::*;

    #[test]
    fn test_obtain_wf_files() -> Result<()> {
        let gh_token = env::github_token(&None::<String>)?;
        let primary_wf = raw_url::RawUrl::new(
            &gh_token,
            &Url::parse(
                "https://github.com/suecharo/gh-trs/blob/main/tests/CWL/wf/trimming_and_qc.cwl",
            )?,
            None,
            None,
        )?;
        let files = obtain_wf_files(&gh_token, &primary_wf)?;
        assert_eq!(files.len(), 3);
        Ok(())
    }
}
