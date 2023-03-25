//! Makefile tasks
use cli as cargo_make;
use crate::{
  cli::Cli, 
  generate::*,
};
use cargo_make::{types::*};
use once_cell::{sync::Lazy};
use serde::{Deserialize, Serialize};
use std::{
  collections::HashMap, 
  fs as stdfs,
  io::{Error as IOError},
};
use thiserror::Error;
use toml::{ser::Error as TomlSerError};
/// Just makes a vec of specified items from arguments
macro_rules! vv {
  (strings $($e:expr,)*) => {{vec![$($e.to_string(),)* ]}};
  (dep_names $($e:expr,)*) => {{vec![$(DependencyIdentifier::Name($e.to_string()),)* ]}};
} 

/// The makefile specification
#[derive(Debug, Deserialize, Serialize)]
pub struct MakefileSpec {
  env: MakefileEnv,
  tasks: HashMap<String, Task>,
}
impl TryFrom<&Cli> for MakefileSpec {
  type Error = MakefileGenerationError;
  fn try_from(
    cli: &Cli,
  ) -> Result<Self, Self::Error> {
    MakefileEnv::try_from(cli)
      .and_then(|env| {
        let mut named_tasks = vec![
          NamedTask::make_crate_scaffold_task(),
          NamedTask::make_lib_code_generator_task(None),
          NamedTask::make_lib_code_generator_task(Some(true)),
          NamedTask::make_openapi_cli_install_task(),
          NamedTask::make_output_dir_clean_task(),
          NamedTask::make_output_dir_create_task(),
          NamedTask::make_spec_download_task(),
        ];
        Ok(Self {
          env, 
          tasks: HashMap::from_iter(
            named_tasks.drain(0..).map(|NamedTask { name, task }| (name, task) )
          ),
        })

      })
  }
}
impl MakefileSpec {
  /// Write makefile to makefile 
  pub fn write_to_makefile(
    &self,
    cli: &Cli
  ) -> Result<(), MakefileGenerationError> {
    toml::to_string_pretty(self)
      .map_err(MakefileGenerationError::from)
      .and_then(|toml_string| {
        let output_dir_path = cli.get_output_project_dir();
        let output_file_name = MakefileEnv::MAKEFILE_NAME;
        let output_file_path = output_dir_path.join(output_file_name);
        stdfs::write(&output_file_path, toml_string)?;
        println!("Wrote makefile to `{output_file_path:?}`");
        Ok(())
      })
  }
}

/// Makefile generation errors 
#[derive(Error, Debug, )]
pub enum MakefileGenerationError {
  #[error("Env missing key {0}")] EnvMissingKey(String),
  #[error(transparent)] IOError(#[from]IOError),
  #[error(transparent)] ParameterError(#[from]ParameterError),
  #[error(transparent)] TomlSerError(#[from]TomlSerError),
}
/// Makefile env
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct MakefileEnv{
  pub api_url: EnvValue,
  pub api_name: EnvValue,  
  pub lib_name: EnvValue,
  pub output_dir: EnvValue,
  pub open_api_generator_cli_url: EnvValue,
  pub open_api_generator_cli_subdir: EnvValue,
  pub open_api_generator_cli_path: EnvValue,
  pub open_api_generator_cli_script: EnvValue,
  pub open_api_generator_config_file: EnvValue,
  pub open_api_generator_config_path: EnvValue,
  pub spec_file_name: EnvValue,
  pub spec_file_path: EnvValue,
  pub spec_file_url: EnvValue,
}
impl TryFrom<&Cli> for MakefileEnv {
  type Error = MakefileGenerationError;
  fn try_from(cli: &Cli) -> Result<Self, Self::Error> {
    let Cli{
      site_or_api_name,
      api_url,
      api_spec_url, ..
    } = cli;
    let lib_name = cli.get_lib_name();
    let spec_file_name = cli.try_get_spec_file_name()?;
    let _output_project_dir_string = cli.get_output_project_dir_string();
    Ok(Self{
      api_url: EnvValue::Value(api_url.to_string()),
      api_name: EnvValue::Value(site_or_api_name.to_string()),
      lib_name: EnvValue::Value(lib_name.to_string()),
      output_dir: EnvValue::Value(".".to_string()),
      open_api_generator_cli_subdir: EnvValue::Value(Self::OPEN_API_GENERATOR_CLI_SUBDIR.to_string()),
      open_api_generator_cli_path: EnvValue::Value( "${OPEN_API_GENERATOR_CLI_SUBDIR}/${OPEN_API_GENERATOR_CLI_SCRIPT}".to_string()),
      open_api_generator_cli_script: EnvValue::Value(Self::OPEN_API_GENERATOR_CLI_SCRIPT.to_string()),
      open_api_generator_cli_url: EnvValue::Value(Self::OPEN_API_GENERATOR_CLI_URL.to_string()),
      open_api_generator_config_file: EnvValue::Value(Self::OPEN_API_GENERATOR_CONFIG_FILE.to_string()),
      open_api_generator_config_path: EnvValue::Value("${OPEN_API_GENERATOR_CONFIG_FILE}".to_string()),
      spec_file_name: EnvValue::Value(spec_file_name),
      spec_file_path: EnvValue::Value(r#"${SPEC_FILE_NAME}"#.to_string()),
      spec_file_url: EnvValue::Value(api_spec_url.to_string()),
    })
  }
}
impl MakefileEnv {
  /// Default config file name for OpenAPI Generator
  pub const OPEN_API_GENERATOR_CONFIG_FILE: &'static str = "generator_config.yaml";
  /// Default download url for OpenAPI Generator CLI artifact
  pub const OPEN_API_GENERATOR_CLI_URL: &'static str = "https://raw.githubusercontent.com/OpenAPITools/openapi-generator/master/bin/utils/openapi-generator-cli.sh";
  /// Default OpenAPI Generator CLI local dir
  pub const OPEN_API_GENERATOR_CLI_SUBDIR: &'static str = "bin/openapitools";
  /// Default OpenAPI Generator CLI local executable name
  pub const OPEN_API_GENERATOR_CLI_SCRIPT: &'static str = "openapi-generator-cli";
  /// Default Makefile name
  pub const MAKEFILE_NAME: &'static str = "Makefile.toml";
}

/// A named [Task] specification
#[derive(Deserialize, Serialize)]
pub struct NamedTask{
  pub name: String,
  pub task: Task,
}
impl NamedTask {
  /// Code generator optiosn 
  const CODE_GENERATION_OPTS: Lazy<Vec<String>> = Lazy::new(|| vv![strings
    "generate", 
    "--generator-name", "rust",
    "--output", "${OUTPUT_DIR}",
    "--input-spec", "${SPEC_FILE_PATH}",
    "--config", "${OPEN_API_GENERATOR_CONFIG_PATH}",
    "-Dcolor",
  ]);
  /// Makes a task that scaffolds the crate 
  pub fn make_crate_scaffold_task() -> NamedTask {
    NamedTask { 
      name: "crate-scaffold".to_string(), 
      task: Task {
        description: Some(r#"Setup ${LIB_NAME} project'."#.to_string()),
        dependencies: Some(vv![ dep_names
          "output-dir-create",
          "output-dir-clean",
          "spec-download",
        ]),
        ..Default::default()
      } 
    }
  }
  
  /// Makes a task that generates the code lib from the openapi spec
  pub fn make_lib_code_generator_task(is_dry_run: Option<bool>) -> NamedTask {
    let mut args = Self::CODE_GENERATION_OPTS.clone();
    let mut name = "lib-code-generate".to_string();
    if let Some(true) = is_dry_run {
      args.push("--dry-run".to_string());
      name.push_str("-dry-run"); 
    } 
    NamedTask {
      name,
      task: Task{
        description: Some("Generate ${LIB_NAME} code".to_string()),
        dependencies: Some(vv![dep_names "open-cli-install",]),
        command: Some("${OPEN_API_GENERATOR_CLI_SCRIPT}".to_string()),
        args: Some(args),
        ..Default::default()
      }
    }
  }
  
  /// Makes a task that installs openapi-generator cli artifact
  pub fn make_openapi_cli_install_task() -> NamedTask {
    NamedTask {
      name: "openapi-cli-bash-install".to_string(),
      task: Task {
        description: Some(r#"Install Open API generator CLI'."#.to_string()),
        script: Some(ScriptValue::SingleLine(
          r#"#!/bin/bash
          # enable the downloaded cli artifact file 
          CLI_SUBDIR=$HOME/${OPEN_API_GENERATOR_CLI_SUBDIR}
          CLI_PATH=$HOME/${OPEN_API_GENERATOR_CLI_PATH}
          CLI_SCRIPT=${OPEN_API_GENERATOR_CLI_SCRIPT}
          if [[ ! -s "$HOME/.bash_profile" && -s "$HOME/.profile" ]] ; then
              PROFILE_FILE="$HOME/.profile"
          else 
              PROFILE_FILE="$HOME/.bash_profile"
          fi
          # echo $CLI_SCRIPT
          function check_cli
          {
              source $PROFILE_FILE
              if command -v $CLI_SCRIPT >& /dev/null
              then 
                  echo "Install success. You can now run the \"$CLI_SCRIPT\" command"
                  echo "After running \"source $PROFILE_FILE\""
                  exit 0
              else 
                  echo "Install failed."
                  exit 0
              fi
          }
          function enable_cli
          {
              chmod u+x $CLI_PATH
              line_to_add="export PATH=\$PATH:$CLI_SUBDIR/"
              if ! grep -q "$line_to_add" "${PROFILE_FILE}" ; then 
                  echo "Adding \"$line_to_add\" to ${PROFILE_FILE}."
                  echo "\n # OpenAPI Generator CLI" >> $PROFILE_FILE
                  echo "$line_to_add" >> $PROFILE_FILE
              else 
                  echo "Line already found in $PROFILE_FILE"
              fi
              check_cli
          } 
          # review the downloaded cli artifact file and optionally enable 
          function deal_with_cli 
          {
              echo Downloaded Open API Generator CLI script at $CLI_PATH
              echo Do you want to enable, review the script or delete it?
              select erd in "Enable" "Review" "Delete"; do
                  case $erd in
                      Enable) 
                          enable_cli
                          break
                          ;;
                      Review)
                          less $CLI_PATH
                          deal_with_cli
                          break
                          ;;
                      Delete)
                          rm $CLI_PATH
                          rm -rf $CLI_SUBDIR
                          exit 1
                          ;;
                  esac
              done 
          }
          # get the cli
          function get_cli 
          {
              mkdir -p $CLI_SUBDIR
              wget -N ${OPEN_API_GENERATOR_CLI_URL} -O $CLI_PATH
          }
  
          get_cli
          deal_with_cli
          "#.to_string()
        )),
        ..Default::default()
      }
    }
  }
  
  /// Makes a task that cleans a library directory
  pub fn make_output_dir_clean_task() -> NamedTask {
    NamedTask {
      name: "output-dir-clean".to_string(),
      task: Task {
        description: Some(r#"Setup ${LIB_NAME} output dir at ${OUTPUT_DIR}'."#.to_string()),
        command: Some("rm".to_string()),
        args: Some(vv![strings "-rf", "${OUTPUT_DIR}/*", ]),
        ..Default::default()
      }
    }
  }
  
  /// Makes a task that sets up a library directory
  pub fn make_output_dir_create_task() -> NamedTask {
    NamedTask {
      name: "output-dir-create".to_string(),
      task: Task {
        description: Some(r#"Create ${LIB_NAME} output dir at ${OUTPUT_DIR}'."#.to_string()),
        command: Some("mkdir".to_string()),
        args: Some(vv![strings  "-p", "${OUTPUT_DIR}",  ]),
        ..Default::default()
      }
    }
  }
  
  /// Makes a task that downloads spec
  pub fn make_spec_download_task() -> NamedTask {
    NamedTask { 
      name: "spec-download".to_string(), 
      task: Task {
        description: Some(r#"Downloads ${API_NAME} Open API specification from '${API_URL}'."#.to_string()),
        command: Some("wget".to_string()),
        args: Some(vv![ strings "-N", "${SPEC_FILE_NAME}", "-O", "${SPEC_FILE_PATH}", ]),
        ..Default::default()
      }
    }
  }
}


// /// A task that uses the openapi spec to generate code
// pub const CodeGeneratorTask: Lazy<Task> = Lazy::new(|| {
//   Task {
//     description: Some(r#"Generates ${API_NAME} Rust code"#.to_string()),
//     ..Default::default()
//   }
// });
// /// A task that uses the open api spec to generate documentation
// pub const DocumentationGeneratorTask: Lazy<Task> = Lazy::new(|| {
//   Task {
//     ..Default::default()
//   }
// });
// /// A task that uses the open api spec to generate tests 
// pub const TestGeneratorTask: Lazy<Task> = Lazy::new(|| {
//   Task {
//     ..Default::default()
//   }
// });
// /// A task that runs the generated tests
// pub const TestRunnerTask: Lazy<Task> = Lazy::new(|| {
//   Task {
//     ..Default::default()
//   }
// });