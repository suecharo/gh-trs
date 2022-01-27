use anyhow::Result;
use std::path::Path;

#[cfg(not(tarpaulin_include))]
pub fn validate(
    _config_file: impl AsRef<Path>,
    _github_token: &Option<impl AsRef<str>>,
) -> Result<()> {
    Ok(())
}
