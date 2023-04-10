//! Makefile tasks
use crate::{
  cli::{Cli, InnerCli, Paths, SubCommands},
  fs,
  generate::{errors::*, utils::*, *},
  vv,
};
use cargo_make::types::*;
use cli as cargo_make;
use futures::TryFutureExt;
use once_cell::sync::Lazy;
use proc_macro2::Span;
// use quote::quote;
use serde::{Deserialize, Serialize};
use serde_yaml::Error as SerdeYAMLError;
use std::{collections::HashMap, io::Error as IOError};
use strum::EnumProperty;
use syn::Ident;
use thiserror::Error;
use toml::ser::Error as TomlSerError;

/// The makefile specification
#[derive(Debug, Deserialize, Serialize)]
pub struct MakefileSpec {
  env: MakefileEnv,
  tasks: HashMap<TaskNames, Task>,
}
impl TryFrom<&Cli> for MakefileSpec {
  type Error = MakefileGenerationError;
  fn try_from(cli: &Cli) -> Result<Self, Self::Error> {
    MakefileEnv::try_from(cli).and_then(|env| {
      let mut named_tasks = vec![
        NamedTask::make_cargo_fix_task(),
        NamedTask::make_crate_scaffold_task(),
        NamedTask::make_generate_all_task(cli)?,
        NamedTask::make_lib_code_generator_task(None),
        NamedTask::make_lib_code_generator_task(Some(true)),
        NamedTask::make_openapi_cli_check_task(),
        NamedTask::make_openapi_cli_install_task(),
        NamedTask::make_output_dir_clean_task(),
        NamedTask::make_output_dir_create_task(),
        NamedTask::make_spec_download_task(),
        NamedTask::make_spec_validate_task(),
      ];
      if cli.inner_cli.api_spec_url_opt.is_some() {
        named_tasks.push(NamedTask::make_spec_download_default_task());
      }
      Ok(Self {
        env,
        tasks: HashMap::from_iter(
          named_tasks
            .drain(0..)
            .map(|NamedTask { name, task }| (name, task)),
        ),
      })
    })
  }
}
impl MakefileSpec {
  /// Write makefile to makefile
  pub async fn write_to_makefile(
    &self,
    cli: &Cli,
  ) -> Result<(), MakefileGenerationError> {
    async { toml::to_string_pretty(self).map_err(MakefileGenerationError::from) }
      .and_then(|toml_string| async {
        let output_dir_path = cli.get_output_project_dir();
        let output_file_name = MakefileEnv::MAKEFILE_NAME;
        let output_file_path = output_dir_path.join(output_file_name);
        fs::write(&output_file_path, toml_string, Some("Makefile")).await?;
        // println!("Wrote makefile to `{output_file_path:?}`");
        Ok(())
      })
      .await
  }
}

/// Makefile generation errors
#[derive(Error, Debug)]
pub enum MakefileGenerationError {
  #[error("Env missing key {0}")]
  EnvMissingKey(String),
  #[error(transparent)]
  IOError(#[from] IOError),
  #[error(transparent)]
  CargoConfigError(#[from] CargoConfigError),
  #[error(transparent)]
  ParameterError(#[from] ParameterError),
  #[error(transparent)]
  READMEGenerationError(#[from] READMEGenerationError),
  #[error(transparent)]
  SerdeYAMLError(#[from] SerdeYAMLError),
  #[error(transparent)]
  TomlSerError(#[from] TomlSerError),
}
/// Makefile env
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct MakefileEnv {
  pub api_url: EnvValue,
  pub api_name: EnvValue,
  pub lib_name: EnvValue,
  pub original_output_dir: EnvValue,
  pub output_dir: EnvValue,
  pub output_temp_dir: EnvValue,
  pub open_api_generator_cli_url: EnvValue,
  pub open_api_generator_cli_subdir: EnvValue,
  pub open_api_generator_cli_path: EnvValue,
  pub open_api_generator_cli_script: EnvValue,
  pub open_api_generator_config_file: EnvValue,
  pub open_api_generator_config_path: EnvValue,
  pub spec_file_download_dir: EnvValue,
  pub spec_file_name: EnvValue,
  pub spec_file_path: EnvValue,
  pub spec_file_url: EnvValue,
}
impl TryFrom<&Cli> for MakefileEnv {
  type Error = MakefileGenerationError;
  fn try_from(cli: &Cli) -> Result<Self, Self::Error> {
    let InnerCli {
      site_or_api_name,
      api_url,
      api_spec_url_opt,
      ..
    } = &cli.inner_cli;
    let lib_name = cli.get_lib_name();
    let spec_file_name = cli.try_get_spec_file_name()?;
    let output_project_dir_string = cli.get_output_project_dir_string();
    let output_project_temp_dir_string = Paths::TempDir.get_str("path").expect("must get temp dir");
    let api_spec_url_string = api_spec_url_opt
      .as_ref()
      .map(|api_url| api_url.to_string())
      .unwrap_or_default();
    Ok(Self {
      api_url: EnvValue::Value(api_url.to_string()),
      api_name: EnvValue::Value(site_or_api_name.to_string()),
      lib_name: EnvValue::Value(lib_name.to_string()),
      original_output_dir: EnvValue::Value(output_project_dir_string),
      output_dir: EnvValue::Script(EnvValueScript {
        script: vv![strings "pwd",],
        multi_line: None,
        condition: None,
        depends_on: None,
      }),
      output_temp_dir: EnvValue::Value(format!("./{output_project_temp_dir_string}")),
      open_api_generator_cli_subdir: EnvValue::Value(
        Self::OPEN_API_GENERATOR_CLI_SUBDIR.to_string(),
      ),
      open_api_generator_cli_path: EnvValue::Value(
        "${OPEN_API_GENERATOR_CLI_SUBDIR}/${OPEN_API_GENERATOR_CLI_SCRIPT}".to_string(),
      ),
      open_api_generator_cli_script: EnvValue::Value(
        Self::OPEN_API_GENERATOR_CLI_SCRIPT.to_string(),
      ),
      open_api_generator_cli_url: EnvValue::Value(Self::OPEN_API_GENERATOR_CLI_URL.to_string()),
      open_api_generator_config_file: EnvValue::Value(
        Self::OPEN_API_GENERATOR_CONFIG_FILE.to_string(),
      ),
      open_api_generator_config_path: EnvValue::Value(
        "${OPEN_API_GENERATOR_CONFIG_FILE}".to_string(),
      ),
      spec_file_download_dir: EnvValue::Value("${OUTPUT_TEMP_DIR}/specdl".to_string()),
      spec_file_name: EnvValue::Value(spec_file_name),
      spec_file_path: EnvValue::Value(r#"${SPEC_FILE_NAME}"#.to_string()),
      spec_file_url: EnvValue::Value(api_spec_url_string),
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
  /// Default Makefile task category
  pub const DEFAULT_TASK_CATEGORY: Lazy<String> =
    Lazy::new(|| format!("{} {}", get_this_crate_name(), get_this_crate_ver_pretty()));
}

/// A named [Task] specification
#[derive(Deserialize, Serialize)]
pub struct NamedTask {
  pub name: TaskNames,
  pub task: Task,
}
impl NamedTask {
  /// Code generator options
  const CODE_GENERATION_OPTS: Lazy<Vec<String>> = Lazy::new(|| {
    vv![strings
      "generate",
      "--generator-name", "rust",
      "--output", "${OUTPUT_DIR}",
      "--input-spec", "${SPEC_FILE_PATH}",
      "--config", "${OPEN_API_GENERATOR_CONFIG_PATH}",
      // "-Dcolor",
    ]
  });

  /// Get default category
  pub fn default_category() -> Option<String> {
    Some(MakefileEnv::DEFAULT_TASK_CATEGORY.to_string())
  }

  /// Makes a task that does cargo fix  
  pub fn make_cargo_fix_task() -> Self {
    Self {
      name: TaskNames::CargoFixGenerated,
      task: Task {
        category: Self::default_category(),
        description: Some(r#"Fix ${LIB_NAME} project generated code'."#.to_string()),
        command: Some("cargo".to_string()),
        args: Some(vv![strings
          "fix",
          "--broken-code",
          "--edition",
          "--edition-idioms",
          "--allow-dirty",
          "--all-targets",
          "--all-features",
          "--verbose",
          "--verbose",
        ]),
        ..Default::default()
      },
    }
  }
  /// Makes a task that scaffolds the crate
  pub fn make_crate_scaffold_task() -> Self {
    Self {
      name: TaskNames::CrateScaffold,
      task: Task {
        category: Self::default_category(),
        description: Some(r#"Setup ${LIB_NAME} project'."#.to_string()),
        dependencies: Some(vv![ as_ref dep_names
          TaskNames::OutputDirCreate,
          TaskNames::OutputDirClean,
        ]),
        ..Default::default()
      },
    }
  }

  /// Makes a task that does all of the generation steps
  pub fn make_generate_all_task(cli: &Cli) -> Result<Self, MakefileGenerationError> {
    // dbg!(cli);
    let name = TaskNames::GenerateAll;
    let cargo_configurator = cargos::CargoConfigurator::new(cli)?;
    let cargo_configurator_yaml = serde_yaml::to_string(&cargo_configurator)?;
    let readme_generator = readmes::READMEGenerator::new(cli)?;
    let readme_generator_yaml = serde_yaml::to_string(&readme_generator)?;
    let this_crate_name = cargo_configurator.this_crate_name.to_string();
    let this_crate_ident = Ident::new(&this_crate_name, Span::call_site());
    let this_crate_ver = cargo_configurator.this_crate_ver.to_string();
    // let cargo_toml_path  = Paths::CargoTomlFile.get_str("path").expect("must get cargo toml path");
    let default_crate_dependency_string =
      format!("{this_crate_name} = {{ version = \"{this_crate_ver}\" }}");
    let this_crate_dependency_string = if let Some(SubCommands::TestGeneration {
      generator_crate_local_path_opt: generator_path_opt,
      generator_crate_repo_url_opt: generator_repo_opt,
      ..
    }) = &cli.inner_cli.command
    {
      match generator_path_opt {
        Some(generator_path) => {
          let this_crate_path_str = generator_path.to_string_lossy();
          format!(
            "{this_crate_name} = \
            {{ version = \"{this_crate_ver}\", \
            path = \"{this_crate_path_str}\" }} "
          )
        }
        None => match generator_repo_opt {
          Some(generator_repo) => {
            let this_crate_repo_str = generator_repo.as_str();
            format!(
              "{this_crate_name} = \
                {{ version = \"{this_crate_ver}\", \
                git = \"{this_crate_repo_str}\" }} "
            )
          }
          None => default_crate_dependency_string,
        },
      }
    } else {
      default_crate_dependency_string
    };
    let mut script_lines = trim_lines_vec(&format!(
      r#"
      //! ```cargo
      //! [dependencies]
      //! {this_crate_dependency_string}
      //! serde_yaml = {{ version = "0.9.19" }}
      //! tokio = {{ version = "1.26.0", features = ["full"] }}
      //! ```
    "#
    ));
    let mut script_body = trim_lines_vec(&format! {r#"
      use {this_crate_ident}::{{ 
        cli::CLIError, 
        generate::{{
          cargos::CargoConfigurator,
          readmes::READMEGenerator 
        }} 
      }};
      mod yaml_specs {{
        pub const CARGO_CONFIGURATOR_YAML: &'static str = "{cargo_configurator_yaml}";
        pub const README_GENERATOR_YAML: &'static str = "{readme_generator_yaml}";
      }}
      #[tokio::main]
      async fn main() -> Result<(), CLIError> {{
        let cargo_configurator: CargoConfigurator = serde_yaml::from_str(yaml_specs::CARGO_CONFIGURATOR_YAML)?;
        cargo_configurator.update_cargo_manifest_post_generation().await?;
        cargo_configurator.update_cargo_manifest_post_fix_edition().await?;
        let readme_generator: READMEGenerator = serde_yaml::from_str(yaml_specs::README_GENERATOR_YAML)?;
        readme_generator.update_readme_md_file().await?;
        println!("updates complete");
        Ok(())
      }}
    "#});
    script_lines.extend(script_body.drain(0..));
    Ok(Self {
      name,
      task: Task {
        category: Self::default_category(),
        description: Some("Generate ${LIB_NAME} code and try to get it up to par".to_string()),
        dependencies: Some(vv![as_ref dep_names
          TaskNames::LibCodeGenerate,
          TaskNames::CargoFixGenerated,
        ]),
        script_runner: Some("@rust".to_string()),
        script: Some(ScriptValue::Text(script_lines)),
        // run_task: Some(RunTaskInfo::Routing(vec![
        //   Self::make_named_run_task_routing_info("cargo-fix-generated", None),
        // ])),
        ..Default::default()
      },
    })
  }

  /// Makes a task that generates the code lib from the openapi spec
  pub fn make_lib_code_generator_task(is_dry_run: Option<bool>) -> Self {
    let mut args = Self::CODE_GENERATION_OPTS.clone();
    let mut name = TaskNames::LibCodeGenerate;
    if let Some(true) = is_dry_run {
      args.push("--dry-run".to_string());
      name = TaskNames::LibCodeGenerateDryRun;
    }
    Self {
      name,
      task: Task {
        category: Self::default_category(),
        description: Some("Generate ${LIB_NAME} code".to_string()),
        condition_script: Some(trim_lines_vec(
          r#"
          #!/bin/bash
          # check if openapi cli command exists
          if command -v ${OPEN_API_GENERATOR_CLI_SCRIPT} >& /dev/null ;  then
            echo "Found OpenAPI CLI command."
            exit 0
          else 
            echo "Missing OpenAPI CLI command. Try running `cargo make openapi-cli-bash-install`"
            exit 1
          fi
          "#,
        )),
        command: Some("${OPEN_API_GENERATOR_CLI_SCRIPT}".to_string()),
        args: Some(args),
        ..Default::default()
      },
    }
  }

  /// Makes a named [routing info](RunTaskRoutingInfo)
  #[allow(dead_code)]
  fn make_named_run_task_routing_info(
    name: &str,
    cleanup_task_opt: Option<&str>,
  ) -> RunTaskRoutingInfo {
    RunTaskRoutingInfo {
      name: RunTaskName::Single(name.to_string()),
      fork: Some(false),
      parallel: Some(false),
      cleanup_task: cleanup_task_opt.map(|s| s.to_string()),
      condition: None,
      condition_script: None,
    }
  }

  /// Makes a task that checks openapi-generator cli artifact
  pub fn make_openapi_cli_check_task() -> Self {
    Self {
      name: TaskNames::OpenapiCliCheck,
      task: Task {
        category: Self::default_category(),
        description: Some("Check that openapi cli generator tool is installed".to_string()),
        command: Some("command".to_string()),
        args: Some(vv![strings "-v", "${OPEN_API_GENERATOR_CLI_SCRIPT}",]),
        ..Default::default()
      },
    }
  }

  /// Makes a task that installs openapi-generator cli artifact
  pub fn make_openapi_cli_install_task() -> Self {
    Self {
      name: TaskNames::OpenapiCliBashInstall,
      task: Task {
        category: Self::default_category(),
        description: Some(r#"Install Open API generator CLI'."#.to_string()),
        script: Some(ScriptValue::Text(trim_lines_vec(
          r#"
          #!/bin/bash
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
                  echo "\# OpenAPI Generator CLI" >> $PROFILE_FILE
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
          "#,
        ))),
        ..Default::default()
      },
    }
  }

  /// Makes a task that cleans a library directory
  pub fn make_output_dir_clean_task() -> Self {
    Self {
      name: TaskNames::OutputDirClean,
      task: Task {
        category: Self::default_category(),
        description: Some(r#"Setup ${LIB_NAME} output dir at ${OUTPUT_DIR}'."#.to_string()),
        command: Some("rm".to_string()),
        args: Some(vv![strings "-rf", "${OUTPUT_DIR}/*", ]),
        ..Default::default()
      },
    }
  }

  /// Makes a task that sets up a library directory
  pub fn make_output_dir_create_task() -> Self {
    Self {
      name: TaskNames::OutputDirCreate,
      task: Task {
        category: Self::default_category(),
        description: Some(r#"Create ${LIB_NAME} output dir at ${OUTPUT_DIR}'."#.to_string()),
        command: Some("mkdir".to_string()),
        // args: Some(vv![strings  "-p", "${OUTPUT_DIR}",  ]),
        ..Default::default()
      },
    }
  }

  /// Makes a task that downloads default spec if known
  pub fn make_spec_download_default_task() -> Self {
    Self {
      name: TaskNames::SpecDownloadDefault,
      task: Task {
        category: Self::default_category(),
        description: Some(
          r#"Downloads ${API_NAME} Open API specification from '${API_URL}'."#.to_string(),
        ),
        command: Some("wget".to_string()),
        args: Some(vv![ strings "${SPEC_FILE_URL}", "-O", "${SPEC_FILE_PATH}", ]),
        ..Default::default()
      },
    }
  }
  /// Makes a task that downloads spec if known
  pub fn make_spec_download_task() -> Self {
    Self {
      name: TaskNames::SpecDownload,
      task: Task {
        category: Self::default_category(),
        description: Some(
          r#"Downloads ${API_NAME} Open API specification from specified vararg'."#.to_string(),
        ),
        command: Some("wget".to_string()),
        args: Some(vv![ strings "${@}", "-O", "${SPEC_FILE_PATH}", ]),
        ..Default::default()
      },
    }
  }
  /// Makes a task that validates the spec
  pub fn make_spec_validate_task() -> Self {
    let name = TaskNames::SpecValidate;
    let description = Some(format!("{name}"));
    Self {
      name,
      task: Task {
        category: Self::default_category(),
        description,
        command: Some("${OPEN_API_GENERATOR_CLI_SCRIPT}".to_string()),
        args: Some(vv![strings "validate", "--input-spec", "${SPEC_FILE_PATH}", "--recommend",]),
        ..Default::default()
      },
    }
  }
}

/// Names of tasks
#[derive(
  Clone,
  Copy,
  Debug,
  Deserialize,
  Error,
  Eq,
  Hash,
  Ord,
  PartialEq,
  PartialOrd,
  Serialize,
  strum::AsRefStr,
)]
#[strum(serialize_all = "kebab-case")]
#[serde(rename_all = "kebab-case")]
pub enum TaskNames {
  #[error("A task that does cargo fix")]
  CargoFixGenerated,
  #[error("A task that scaffolds the crate")]
  CrateScaffold,
  #[error("A task that does all of the generation steps ")]
  GenerateAll,
  #[error("A task that generates the code lib from the openapi spec")]
  LibCodeGenerate,
  #[error("A task that does a DRY RUN of generating the code lib from the openapi spec")]
  LibCodeGenerateDryRun,
  #[error("A task that checks openapi-generator cli artifact")]
  OpenapiCliCheck,
  #[error("A task that installs openapi-generator cli artifact")]
  OpenapiCliBashInstall,
  #[error("A task that cleans a library directory")]
  OutputDirClean,
  #[error("A task that sets up a library directory")]
  OutputDirCreate,
  #[error("A task that downloads default spec if known")]
  SpecDownloadDefault,
  #[error("A task that downloads spec if known")]
  SpecDownload,
  #[error("A task that validates the spec")]
  SpecValidate,
}
