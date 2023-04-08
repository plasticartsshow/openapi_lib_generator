//! CLI Data
use chrono::{DateTime, Utc};
use clap::{
  Parser, 
  Subcommand,
}; 
use crate::{
  generate::{parameters, errors::*, utils, yamls}, 
  testing::{TestingError}
};
use once_cell::{sync::Lazy};
use serde::{Deserialize, Serialize};
use serde_yaml::{Error as SerdeYAMLError};
use std::{
  env,
  ops::{Deref},
  path::{PathBuf},
};
use strum::{EnumProperty};
use thiserror::Error;
use url::Url;

/// Defaults 
mod defaults {
  use super::*;
  /// Current working directory
  pub const CWD: Lazy<PathBuf> = Lazy::new(|| env::current_dir().expect("must get current dir"));
}
use defaults::*;
 

#[derive(Clone, Deserialize, Serialize)]
pub struct Cli{
  pub inner_cli: InnerCli,
  pub generation_timestamp: DateTime<Utc>,
}
impl Deref for Cli {
  type Target = InnerCli;
  fn deref(&self) -> &Self::Target { &self.inner_cli }
}
impl Cli {
  /// Get formatted timestamp string (RFC 3339)
  pub fn get_generation_timestamp_string(&self) -> String {
    self.generation_timestamp.to_rfc3339()
  }
  /// Instantiate 
  pub async fn new() -> Result<Self, CLIError> {
    let mut inner_cli = InnerCli::parse();
    let InnerCli {
      command, 
      output_project_dir_opt, 
      local_api_spec_filepath_opt, 
      ..
    } = &mut inner_cli;
    if let Some(SubCommands::TestGeneration { 
      // generator_crate_local_path_opt, 
      // generator_crate_repo_url_opt ,
      ..
    }) = command.as_mut() {
      // use the temp directory 
      let temp_root_path = utils::get_temp_root_dir();
      let temp_subdir_path = utils::get_temp_subdir();
      if local_api_spec_filepath_opt.is_none() {
        let yaml_test_spec_path = yamls::create_testing_spec_file(&temp_root_path).await?;
        let _ = local_api_spec_filepath_opt.replace(yaml_test_spec_path);
      }
      if output_project_dir_opt.is_none() {
        let _ = output_project_dir_opt.replace(temp_subdir_path);
      }
    } else if inner_cli.local_api_spec_filepath_opt.is_none() && inner_cli.api_spec_url_opt.is_none() {
      Err(ParameterError::APIUrlNeededIfNoLocalFile.into())
    }
    Ok(Self {
      generation_timestamp: Utc::now(),
      inner_cli
    })
  }
}

/// CLI Errors
#[derive(Error, Debug, )]
pub enum CLIError {
  #[error(transparent)]CargoConfigError(#[from] CargoConfigError),
  #[error(transparent)]CrateScaffoldingError(#[from] CrateScaffoldingError),
  #[error(transparent)]MakefileGenerationError(#[from] MakefileGenerationError),
  #[error(transparent)]ParameterError(#[from] ParameterError),
  #[error(transparent)]READMEGenerationError(#[from] READMEGenerationError),
  #[error(transparent)]SerdeYAMLError(#[from] SerdeYAMLError),
  #[error(transparent)]YAMLGenerationError(#[from] YAMLGenerationError),
  #[error(transparent)]TestingError(#[from] TestingError),
}


/// Subcommands for the [InnerCli]
#[derive(Clone, Deserialize, Serialize, Subcommand)]
pub enum SubCommands {
  /// Tests code generation 
  /// 
  /// You MAY provide EITHER of a generator path or a generator repo url pointing to the generator crate 
  #[command(rename_all = "kebab-case", verbatim_doc_comment)]
  TestGeneration {
    /// This is the path to the crate THIS CLI came from
    #[arg(
      short = 'p',
      long = "generator-crate-local-path",  
      required_unless_present("generator_crate_repo_url_opt"),
    )]
    generator_crate_local_path_opt: Option<PathBuf>,
    /// This is the URL to the git repo THIS CLI should come from
    #[arg(
      short = 'u',
      long = "generator-crate-repo-url",
    )]
    generator_crate_repo_url_opt: Option<Url>,
  }
}

/// =================== OpenAPI client  crate generator ====================
/// 
///  ___     ___     //=//  ___      /----\     ___  \\=\\     ___     ___
/// /  /    /  /    //  \\  \  \    /  /\  \   /  /  //  \\    \  \    \  \
/// |  \___/   \___// /\ \\__\  \__|  |__|  |_/  /__// /\ \\___/   \___/   |
///  \_______/\______/  \_____________________________/  \______/\________/ 
/// Generate a client crate for the given OpenAPI—compliant web application.  
/// The specifications must be provided either as a url or a local file.   
/// ||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
#[derive(Clone, Deserialize, Parser, Serialize)]
#[command(author, version, about, verbatim_doc_comment )]
pub struct InnerCli {
  /// The site or app name. Will be used to determine generated crate name
  #[arg(long="name")]
  pub site_or_api_name: String, 
  /// The app URL. It's just there for documentation and referencing.
  #[arg(long="api-url")]
  pub api_url: Url,
  /// The api spec URL. If provided, the generator will fetch the json or yaml OpenAPI specification from here.
  #[arg(long="spec-url")]
  pub api_spec_url_opt: Option<Url>, 
  /// API Spec as a local file. If provided, this overrides the [Self::api_spec_url_opt]
  #[arg(long="spec-file")]
  pub local_api_spec_filepath_opt: Option<PathBuf>,
  /// Optional library name to override default generated crate name
  #[arg(long="lib_name")]
  pub lib_name_opt: Option<String>,
  /// Optional `;`—separated extra authors to add to list
  #[arg(long="authors")]
  pub extra_authors: Option<String>,
  /// The optional output project dir
  #[arg(long="output")]
  output_project_dir_opt: Option<PathBuf>,
  #[command(subcommand)]
  pub command: Option<SubCommands>,
}
impl InnerCli {
  /// Temp dir 
  pub const TEMP_DIR_NAME: &'static str = "temp";
  /// Get  authors strings
  pub fn get_extra_authors(&self) -> Vec<String> {
    Self::parse_authors_string(&self.extra_authors.clone().unwrap_or_default())
  }
  /// Get a default project library name 
  fn get_default_lib_name(&self) -> String {
    let Self {site_or_api_name, ..} = self;
    format!("{site_or_api_name}_api_lib")
  }
  /// Get a default project spec file name 
  fn get_default_spec_file_name(&self) -> String {
    let mut name_path = PathBuf::from(self.get_lib_name());
    name_path.set_extension("yaml");
    name_path.to_string_lossy().to_string()
  }
  /// Get the project library name 
  pub fn get_lib_name(&self) -> String {
    self.lib_name_opt.clone()
    .unwrap_or_else(|| self.get_default_lib_name())
  }
  
  /// Get the output project dir
  pub fn get_output_project_dir(&self) -> PathBuf {
    self.output_project_dir_opt.clone()
      .unwrap_or_else(|| CWD.clone())
  }
    
  /// Get the output project dir
  pub fn get_output_project_dir_string(&self) -> String {
    self.get_output_project_dir().to_string_lossy().to_string()
  }
  /// Get a subpath in the main dir
  pub fn get_output_project_subpath(&self, subpath: &Paths) -> PathBuf { 
    self.get_output_project_dir()
      .join(subpath.get_str("path").expect("must get subpath")) 
  }
  /// Get a subpath in the main dir as a string 
  pub fn get_output_project_subpath_string(&self, subpath: &Paths) -> String { 
    self.get_output_project_subpath(subpath).to_string_lossy().to_string()
  }
  /// Parse  an authors string 
  pub fn parse_authors_string(s: &str) -> Vec<String> {
    s.split(";").map(|s| s.to_string()).collect()
  }
  /// Get spec file name as specified by [Self::api_spec_url]
  pub fn try_get_spec_file_name(&self) -> Result<String, ParameterError> {
    if let Some(local_api_spec_filepath) = self.local_api_spec_filepath_opt.as_ref() {
      Ok(local_api_spec_filepath.to_string_lossy().to_string())
    } else {
      let api_spec_url = self.api_spec_url_opt.clone().expect("must get spec url");
      parameters::try_file_name_from_path_url(&api_spec_url)
        .map(|mut s| {
          if s.is_empty() { 
            s.push_str(&self.get_default_spec_file_name()) 
          }
          s
        })

    }
  }
}



/// Common Paths 
#[derive(Clone, Copy, Debug, Error, strum::EnumProperty)]
pub enum Paths {
  #[error(".gitignore file")] #[strum(props(path = ".gitignore"))] GitignoreFile,
  #[error(".git dir")] #[strum(props(path = ".git"))] GitDir,
  #[error("Cargo Make make file")] #[strum(props(path = "Makefile.toml"))] CargoMakefile,
  #[error("Cargo.toml file")] #[strum(props(path = "Cargo.toml"))] CargoTomlFile,
  #[error("README.md file")] #[strum(props(path = "README.md"))] ReadmeMdFile,
  #[error("temp dir")] #[strum(props(path = "temp"))] TempDir,
}