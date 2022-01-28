use anyhow::Result;
use std::path::Path;

#[cfg(not(tarpaulin_include))]
pub fn validate(
    config_file: impl AsRef<Path>,
    github_token: &Option<impl AsRef<str>>,
) -> Result<()> {
    Ok(())
}
