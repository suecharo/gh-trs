use crate::config;
use crate::remote;
use crate::trs;
use crate::trs_api;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct TrsResponse {
    pub gh_trs_config: config::Config,
    pub service_info: trs::ServiceInfo,
    pub tool_classes: Vec<trs::ToolClass>,
    pub tools: Vec<trs::Tool>,
    pub tools_id: trs::Tool,
    pub tools_id_versions: Vec<trs::ToolVersion>,
    pub tools_id_versions_version: trs::ToolVersion,
    pub tools_id_versions_version_descriptor: trs::FileWrapper,
    pub tools_id_versions_version_files: Vec<trs::ToolFile>,
    pub tools_id_versions_version_tests: Vec<trs::FileWrapper>,
    pub tools_id_versions_version_containerfile: Vec<trs::FileWrapper>,
}

impl TrsResponse {
    pub fn new(
        config: &config::Config,
        owner: impl AsRef<str>,
        name: impl AsRef<str>,
        verified: bool,
    ) -> Result<Self> {
        let service_info = trs::ServiceInfo::new_or_update(
            trs_api::get_service_info(&owner, &name).ok(),
            &config,
            &owner,
            &name,
        )?;
        let tool_classes = generate_tool_classes(owner.as_ref(), name.as_ref())?;

        let mut tools = match trs_api::get_tools(&owner, &name) {
            Ok(tools) => tools,
            Err(_) => vec![],
        };
        let tools_id = match tools.iter().find(|t| t.id == config.id) {
            Some(tool) => {
                // update tool
                let mut tool = tool.clone();
                tool.add_new_tool_version(&config, &owner, &name, verified)?;
                tools = tools
                    .into_iter()
                    .filter(|t| t.id != config.id)
                    .collect::<Vec<trs::Tool>>();
                tools.push(tool.clone());
                tool
            }
            None => {
                // create tool and add
                let mut tool = trs::Tool::new(&config, &owner, &name)?;
                tool.add_new_tool_version(&config, &owner, &name, verified)?;
                tools.push(tool.clone());
                tool
            }
        };
        let tools_id_versions = tools_id.versions.clone();
        let tools_id_versions_version = tools_id_versions
            .iter()
            .find(|tv| tv.version() == config.version)
            .unwrap() // already created
            .clone();

        let tools_id_versions_version_descriptor = generate_descriptor(&config)?;
        let tools_id_versions_version_files = generate_tools_id_versions_version_files(&config)?;
        let tools_id_versions_version_tests = generate_tools_id_versions_version_tests(&config)?;

        Ok(Self {
            gh_trs_config: config.clone(),
            service_info,
            tool_classes,
            tools,
            tools_id,
            tools_id_versions,
            tools_id_versions_version,
            tools_id_versions_version_descriptor,
            tools_id_versions_version_files,
            tools_id_versions_version_tests,
            tools_id_versions_version_containerfile: vec![],
        })
    }

    pub fn generate_contents(&self) -> Result<HashMap<PathBuf, String>> {
        let id = self.tools_id.id.clone();
        let version = self.tools_id_versions_version.version().clone();
        let descriptor_type = self
            .gh_trs_config
            .workflow
            .language
            .r#type
            .clone()
            .unwrap()
            .to_string();
        let mut map: HashMap<PathBuf, String> = HashMap::new();
        map.insert(
            PathBuf::from(format!(
                "tools/{}/versions/{}/gh-trs-config.json",
                id, version
            )),
            serde_json::to_string(&self.gh_trs_config)?,
        );
        map.insert(
            PathBuf::from("service-info/index.json"),
            serde_json::to_string(&self.service_info)?,
        );
        map.insert(
            PathBuf::from("toolClasses/index.json"),
            serde_json::to_string(&self.tool_classes)?,
        );
        map.insert(
            PathBuf::from("tools/index.json"),
            serde_json::to_string(&self.tools)?,
        );
        map.insert(
            PathBuf::from(format!("tools/{}/index.json", id)),
            serde_json::to_string(&self.tools_id)?,
        );
        map.insert(
            PathBuf::from(format!("tools/{}/versions/index.json", id)),
            serde_json::to_string(&self.tools_id_versions)?,
        );
        map.insert(
            PathBuf::from(format!("tools/{}/versions/{}/index.json", id, version)),
            serde_json::to_string(&self.tools_id_versions_version)?,
        );
        map.insert(
            PathBuf::from(format!(
                "tools/{}/versions/{}/{}/descriptor/index.json",
                id, version, descriptor_type
            )),
            serde_json::to_string(&self.tools_id_versions_version_descriptor)?,
        );
        map.insert(
            PathBuf::from(format!(
                "tools/{}/versions/{}/{}/files/index.json",
                id, version, descriptor_type
            )),
            serde_json::to_string(&self.tools_id_versions_version_files)?,
        );
        map.insert(
            PathBuf::from(format!(
                "tools/{}/versions/{}/{}/tests/index.json",
                id, version, descriptor_type
            )),
            serde_json::to_string(&self.tools_id_versions_version_tests)?,
        );
        map.insert(
            PathBuf::from(format!(
                "tools/{}/versions/{}/containerfile/index.json",
                id, version
            )),
            serde_json::to_string(&self.tools_id_versions_version_containerfile)?,
        );
        Ok(map)
    }
}

fn generate_tool_classes(owner: &str, name: &str) -> Result<Vec<trs::ToolClass>> {
    match trs_api::get_tool_classes(&owner, &name) {
        Ok(mut tool_classes) => {
            let has_workflow = tool_classes
                .iter()
                .find(|tc| tc.id == Some("workflow".to_string()));
            if has_workflow.is_none() {
                tool_classes.push(trs::ToolClass::default());
            };
            Ok(tool_classes)
        }
        Err(_) => Ok(vec![trs::ToolClass::default()]),
    }
}

fn generate_descriptor(config: &config::Config) -> Result<trs::FileWrapper> {
    let primary_wf = config.workflow.primary_wf()?;
    let (content, checksum) = match remote::fetch_raw_content(&primary_wf.url) {
        Ok(content) => {
            let checksum = trs::Checksum::new_from_string(content.clone());
            (Some(content), Some(vec![checksum]))
        }
        Err(_) => (None, None),
    };
    Ok(trs::FileWrapper {
        content,
        checksum,
        url: Some(primary_wf.url),
    })
}

fn generate_tools_id_versions_version_files(config: &config::Config) -> Result<Vec<trs::ToolFile>> {
    Ok(config
        .workflow
        .files
        .iter()
        .map(|f| {
            let checksum = match trs::Checksum::new_from_url(&f.url) {
                Ok(checksum) => Some(checksum),
                Err(_) => None,
            };
            trs::ToolFile {
                path: Some(f.url.clone()),
                file_type: Some(trs::FileType::new_from_file_type(&f.r#type)),
                checksum,
            }
        })
        .collect())
}

fn generate_tools_id_versions_version_tests(
    config: &config::Config,
) -> Result<Vec<trs::FileWrapper>> {
    Ok(config
        .workflow
        .testing
        .iter()
        .map(|t| {
            let test_str = serde_json::to_string(&t)?;
            Ok(trs::FileWrapper {
                content: Some(test_str.clone()),
                checksum: Some(vec![trs::Checksum::new_from_string(test_str)]),
                url: None,
            })
        })
        .collect::<Result<Vec<_>>>()?)
}

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod tests {
    use super::*;
    use crate::config_io;

    #[test]
    fn test_trs_response_new() -> Result<()> {
        let config = config_io::read_config("./tests/test_config_CWL_validated.yml")?;
        TrsResponse::new(&config, "test_owner", "test_name", false)?;
        Ok(())
    }

    #[test]
    fn test_generate_tool_classes() -> Result<()> {
        let tool_classes = generate_tool_classes("test_owner", "test_name")?;
        let expect = serde_json::from_str::<Vec<trs::ToolClass>>(
            r#"
[
  {
    "id": "workflow",
    "name": "Workflow",
    "description": "A computational workflow"
  }
]"#,
        )?;
        assert_eq!(tool_classes, expect);
        Ok(())
    }

    #[test]
    fn test_generate_descriptor() -> Result<()> {
        let config = config_io::read_config("./tests/test_config_CWL_validated.yml")?;
        generate_descriptor(&config)?;
        Ok(())
    }

    #[test]
    fn test_generate_tools_id_versions_version_files() -> Result<()> {
        let config = config_io::read_config("./tests/test_config_CWL_validated.yml")?;
        let files = generate_tools_id_versions_version_files(&config)?;
        let expect = serde_json::from_str::<Vec<trs::ToolFile>>(
            r#"
[
  {
    "path": "https://raw.githubusercontent.com/suecharo/gh-trs/b02f189daddcbc2c0a2c0091300f2b90cca49c49/tests/CWL/wf/fastqc.cwl",
    "file_type": "SECONDARY_DESCRIPTOR",
    "checksum": {
      "checksum": "1bd771a51336a782b695db8334872e00f305cd7c49c4978e7e58786ea4714437",
      "type": "sha256"
    }
  },
  {
    "path": "https://raw.githubusercontent.com/suecharo/gh-trs/b02f189daddcbc2c0a2c0091300f2b90cca49c49/tests/CWL/wf/trimming_and_qc.cwl",
    "file_type": "PRIMARY_DESCRIPTOR",
    "checksum": {
      "checksum": "33ef70b2d5ee38cb394c5ca6354243f44a85118271026eb9fc61365a703e730b",
      "type": "sha256"
    }
  },
  {
    "path": "https://raw.githubusercontent.com/suecharo/gh-trs/b02f189daddcbc2c0a2c0091300f2b90cca49c49/tests/CWL/wf/trimmomatic_pe.cwl",
    "file_type": "SECONDARY_DESCRIPTOR",
    "checksum": {
      "checksum": "531d0a38116347cade971c211056334f7cae48e1293e2bb0e334894e55636f8e",
      "type": "sha256"
    }
  }
]
"#,
        )?;
        assert_eq!(files, expect);
        Ok(())
    }

    #[test]
    fn test_generate_tools_id_versions_version_tests() -> Result<()> {
        let config = config_io::read_config("./tests/test_config_CWL_validated.yml")?;
        let tests = generate_tools_id_versions_version_tests(&config)?;
        let expect = serde_json::from_str::<Vec<trs::FileWrapper>>(
            r#"
[
  {
    "content": "{\"id\":\"test_1\",\"files\":[{\"url\":\"https://raw.githubusercontent.com/suecharo/gh-trs/b02f189daddcbc2c0a2c0091300f2b90cca49c49/tests/CWL/test/wf_params.json\",\"target\":\"wf_params.json\",\"type\":\"wf_params\"},{\"url\":\"https://raw.githubusercontent.com/suecharo/gh-trs/b02f189daddcbc2c0a2c0091300f2b90cca49c49/tests/CWL/test/ERR034597_1.small.fq.gz\",\"target\":\"ERR034597_1.small.fq.gz\",\"type\":\"other\"},{\"url\":\"https://raw.githubusercontent.com/suecharo/gh-trs/b02f189daddcbc2c0a2c0091300f2b90cca49c49/tests/CWL/test/ERR034597_2.small.fq.gz\",\"target\":\"ERR034597_2.small.fq.gz\",\"type\":\"other\"}]}",
    "checksum": [
      {
        "checksum": "89d3af294e5168a27b3c281516e0586db9fb0c0485e49737fb648f9de1165f2f",
        "type": "sha256"
      }
    ]
  }
]
"#,
        )?;
        assert_eq!(tests, expect);
        Ok(())
    }
}
