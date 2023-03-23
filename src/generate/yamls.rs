//! YAML config file generation

use crate::{
  cli::Cli,
  generate::{
    makefiles::{MakefileEnv},
  }
}; 
use serde::{Deserialize, Serialize};
use serde_yaml::{Error as SerdeYAMLError};
use thiserror::Error;
use std::{
  fs::{File}, 
  io::{Error as IOError,}, 
  // path::{Path},
};
/// Errors that can happen with yaml generation
#[derive(Debug, Error)]
pub enum YAMLGenerationError {
  #[error(transparent)] IOError(#[from] IOError),
  #[error(transparent)] SerdeYAMLError(#[from] SerdeYAMLError),
}

/// Rust OpenAPI Generator Configs  
/// 
/// - See: <https://openapi-generator.tech/docs/generators/rust/>
#[derive(Debug, Deserialize, Serialize)]
#[allow(non_snake_case)]
pub struct OpenAPIRustGeneratorConfigs {
  /// Use best fitting integer type where minimum or maximum is set (default false)
  pub bestFitInt: bool,
  /// Suffix that will be appended to all enum names.
  pub enumNameSuffix: String,
  /// Hides the generation timestamp when files are generated. (default true)
  pub hideGenerationTimestamp: bool,
  /// library template (sub-template) to use.(hyper or reqwest, default reqwest)
  pub library: String,
  /// Rust package name (convention: lowercase). (default openapi)
  pub packageName:String,
  /// Rust package version.(default 1.0.0)
  pub packageVersion: String,
  /// Prefer unsigned integers where minimum value is >= 0(default false)
  pub preferUnsignedInt: bool,
  /// If set, generate async function call instead. This option is for 'reqwest' library only(default true)
  pub supportAsync: bool,
  /// If set, add support for reqwest-middleware. This option is for 'reqwest' library only(default false)
  pub supportMiddleware: bool,
  /// If set, return type wraps an enum of all possible 2xx schemas. This option is for 'reqwest' library only (default false)
  pub supportMultipleResponses: bool,
  /// Setting this property to true will generate functions with a single argument containing all API endpoint parameters instead of one argument per parameter.(default false)
  pub useSingleRequestParameter: bool,
  /// Whether to include AWS v4 signature support (default false)
  pub withAWSV4Signature: bool,
}
impl Default for OpenAPIRustGeneratorConfigs {
  fn default() -> Self {
    Self {
      bestFitInt: false,
      enumNameSuffix:  Default::default(),
      hideGenerationTimestamp: true,
      library:  "reqwest".to_string(),
      packageName: "openapi".to_string(),
      packageVersion: "1.0.0".to_string(),
      preferUnsignedInt: false,
      supportAsync: true,
      supportMiddleware: false,
      supportMultipleResponses: false,
      useSingleRequestParameter: false,
      withAWSV4Signature: false,
    }
  }
}
impl OpenAPIRustGeneratorConfigs {
  /// Instantiate 
  pub fn new(cli: &Cli) -> Self {
    Self {
      packageName: cli.get_lib_name(),
      ..Default::default()
    }
  }
  /// Write configs to yaml file 
  pub fn write_to_yaml_file(
    &self, 
    cli: &Cli,
  ) -> Result<(), YAMLGenerationError> {
    let output_dir = cli.get_output_project_dir();
    let output_file_name = MakefileEnv::OPEN_API_GENERATOR_CONFIG_FILE;
    let output_path = output_dir.join(output_file_name); 
    let file = File::create(output_path)?;
    serde_yaml::to_writer(file, self).map_err(From::from)
  }
}