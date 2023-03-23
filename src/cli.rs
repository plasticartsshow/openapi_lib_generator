//! CLI Data
use clap::{
  Parser, 
  // Subcommand,
}; 
use crate::{generate::{parameters}};
use once_cell::{sync::Lazy};
use serde::{Deserialize, Serialize};
use std::{
  env,
  path::{PathBuf},
};
use thiserror::Error;
use url::Url;

/// Defaults 
mod defaults {
  use super::*;
  /// Current working directory
  pub const CWD: Lazy<PathBuf> = Lazy::new(|| env::current_dir().expect("must get current dir"));
}
use defaults::*;

use crate::generate::parameters::ParameterError;


/// CLI Errors
#[derive(Error, Debug, )]
pub enum CLIError {
  #[error(transparent)]CrateScaffoldingError(#[from]crate::generate::crate_scaffolds::CrateScaffoldingError),
  #[error(transparent)]MakefileGenerationError(#[from]crate::generate::makefiles::MakefileGenerationError),
  #[error(transparent)]ParameterError(#[from]crate::generate::parameters::ParameterError),
  #[error(transparent)]YAMLGenerationError(#[from]crate::generate::yamls::YAMLGenerationError),
}

/// CLI Arguments
#[derive(Clone, Deserialize, Parser, Serialize)]
#[command(about = "Rust crate generator using Open API specifications.")]
pub struct Cli {
  /// The site or app name
  #[arg(long="name")]
  pub site_or_api_name: String, 
  /// The app URL 
  #[arg(long="url")]
  pub api_url: Url,
  /// The api spec URL 
  #[arg(long="spec")]
  pub api_spec_url: Url, 
  /// Optional library name 
  pub lib_name_opt: Option<String>,
  /// The optional output project dir
  #[arg(long="output")]
  output_project_dir_opt: Option<PathBuf>,
  // #[command(subcommand)]
  // command: Option<Commands>
}
impl Cli {
  /// Get a default project library name 
  fn get_default_lib_name(&self) -> String {
    let Self {site_or_api_name, ..} = self;
    format!("{site_or_api_name}_api_lib")
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
  /// Get spec file name as specified by [Self::api_spec_url]
  pub fn try_get_spec_file_name(&self) -> Result<String, ParameterError> {
    parameters::try_file_name_from_path_url(&self.api_spec_url)
  }
}

// #[derive(Subcommand)]
// pub enum Commands{
//   /// Manage OpenAPI generator installation
//   Tools {
    
//   }
// }