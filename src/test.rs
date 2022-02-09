use crate::config;
use crate::env;
use crate::wes;

use anyhow::{anyhow, bail, ensure, Result};
use log::{debug, info};
use std::env as std_env;
use std::fs;
use std::io::{BufWriter, Write};
use std::thread;
use std::time;
use url::Url;

pub fn test(config: &config::Config, wes_loc: &Option<Url>, docker_host: &Url) -> Result<()> {
    let wes_loc = match wes_loc {
        Some(wes_loc) => wes_loc.clone(),
        None => {
            wes::start_wes(&docker_host)?;
            Url::parse(&wes::default_wes_location())?
        }
    };
    info!("Use WES location: {} for testing", wes_loc);

    let supported_wes_versions = wes::get_supported_wes_versions(&wes_loc)?;
    ensure!(
        supported_wes_versions
            .into_iter()
            .find(|v| v == "sapporo-wes-1.0.1")
            .is_some(),
        "gh-trs only supports WES version sapporo-wes-1.0.1"
    );

    let in_ci = env::in_ci();
    let mut failed_tests = vec![];
    for test_case in &config.workflow.testing {
        info!("Testing {}", test_case.id);
        let form = wes::test_case_to_form(&config.workflow, test_case)?;
        debug!("Form:\n{:#?}", &form);
        let run_id = wes::post_run(&wes_loc, form)?;
        info!("WES run_id: {}", run_id);
        let mut status = wes::RunStatus::Running;
        while status == wes::RunStatus::Running {
            status = wes::get_run_status(&wes_loc, &run_id)?;
            debug!("WES run status: {:?}", status);
            thread::sleep(time::Duration::from_secs(5));
        }
        let run_log = serde_json::to_string_pretty(&wes::get_run_log(&wes_loc, &run_id)?)?;
        if in_ci {
            let test_log_file =
                std_env::current_dir()?.join(format!("test-logs/{}.log", test_case.id));
            fs::create_dir_all(
                test_log_file
                    .parent()
                    .ok_or(anyhow!("Failed to create dir"))?,
            )?;
            let mut buffer = BufWriter::new(fs::File::create(&test_log_file)?);
            buffer.write(run_log.as_bytes())?;
        }
        match status {
            wes::RunStatus::Complete => {
                info!("Complete {}", test_case.id);
                debug!("Run log:\n{}", run_log);
            }
            wes::RunStatus::Failed => {
                if in_ci {
                    info!("Failed {}", test_case.id);
                    failed_tests.push(test_case.id.clone());
                } else {
                    bail!("Failed {}. Run log:\n{}", test_case.id, run_log);
                }
            }
            _ => {
                unreachable!("WES run status: {:?}", status);
            }
        }
    }

    if failed_tests.len() > 0 {
        bail!(
            "Failed {} tests: {}",
            failed_tests.len(),
            failed_tests.join(", ")
        );
    }

    wes::stop_wes(&docker_host)?;

    Ok(())
}
