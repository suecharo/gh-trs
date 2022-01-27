use anyhow::Result;
use std::path::Path;

pub fn publish(
    _config_file: impl AsRef<Path>,
    _github_token: &Option<impl AsRef<str>>,
    _repo: impl AsRef<str>,
    _branch: impl AsRef<str>,
) -> Result<()> {
    Ok(())
}
