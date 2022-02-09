use std::path::PathBuf;
use structopt::{clap, StructOpt};
use url::Url;

#[derive(StructOpt, Debug, PartialEq, Clone)]
#[structopt(
    name = env!("CARGO_PKG_NAME"),
    about = env!("CARGO_PKG_DESCRIPTION"),
    version = env!("CARGO_PKG_VERSION"),
)]
#[structopt(setting(clap::AppSettings::ColoredHelp))]
pub enum Args {
    /// Make a template for the gh-trs configuration file.
    MakeTemplate {
        /// Location of the primary workflow document.
        workflow_location: Url,

        /// GitHub Personal Access Token.
        #[structopt(long)]
        github_token: Option<String>,

        /// Path to the output file.
        #[structopt(short, long, parse(from_os_str), default_value = "gh-trs-config.yml")]
        output: PathBuf,

        /// Verbose mode.
        #[structopt(short, long)]
        verbose: bool,
    },

    /// Validate the gh-trs configuration file.
    Validate {
        /// Location of the gh-trs configuration file (local file path or remote URL).
        #[structopt(default_value = "gh-trs-config.yml")]
        config_location: String,

        /// GitHub Personal Access Token.
        #[structopt(long)]
        github_token: Option<String>,

        /// Verbose mode.
        #[structopt(short, long)]
        verbose: bool,
    },

    /// Publish the TRS response to GitHub.
    Publish {
        /// Location of the gh-trs configuration file (local file path or remote URL).
        #[structopt(default_value = "gh-trs-config.yml")]
        config_location: String,

        /// GitHub Personal Access Token.
        #[structopt(long)]
        github_token: Option<String>,

        /// GitHub Repository to publish to. (e.g. owner/name)
        #[structopt(short, long, required = true)]
        repo: String,

        /// GitHub branch to publish to.
        #[structopt(short, long, default_value = "gh-pages")]
        branch: String,

        /// Verbose mode.
        #[structopt(short, long)]
        verbose: bool,
    },
}

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn test_make_template() -> Result<()> {
        let args = Args::from_iter(&[
            "gh-trs",
            "make-template",
            "https://github.com/suecharo/gh-trs/blob/main/path/to/workflow.yml",
        ]);
        assert_eq!(
            args,
            Args::MakeTemplate {
                workflow_location: Url::parse(
                    "https://github.com/suecharo/gh-trs/blob/main/path/to/workflow.yml"
                )?,
                github_token: None,
                output: PathBuf::from("gh-trs-config.yml"),
                verbose: false,
            }
        );
        Ok(())
    }

    #[test]
    fn test_validate() -> Result<()> {
        let args = Args::from_iter(&["gh-trs", "validate", "gh-trs-config.yml"]);
        assert_eq!(
            args,
            Args::Validate {
                config_location: "gh-trs-config.yml".to_string(),
                github_token: None,
                verbose: false,
            }
        );
        Ok(())
    }

    #[test]
    fn test_publish() -> Result<()> {
        let args = Args::from_iter(&[
            "gh-trs",
            "publish",
            "gh_trs_config.yml",
            "--repo",
            "suecharo/gh-trs",
        ]);
        assert_eq!(
            args,
            Args::Publish {
                config_location: "gh_trs_config.yml".to_string(),
                repo: "suecharo/gh-trs".to_string(),
                github_token: None,
                branch: "gh-pages".to_string(),
                verbose: false,
            }
        );
        Ok(())
    }
}
