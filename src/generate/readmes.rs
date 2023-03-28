//! README file generation

use crate::{cli::{Cli, InnerCli, Paths}, generate::*}; 
use thiserror::Error;
use tokio::{fs};
use std::{io::{Error as IOError}};
/// Errors that can happen with yaml generation
#[derive(Debug, Error)]
pub enum READMEGenerationError {
  #[error(transparent)] IOError(#[from] IOError),
  // #[error(transparent)] SerdeYAMLError(#[from] SerdeYAMLError),
}

/// Readme generation 
#[derive(Debug,)]
#[allow(non_camel_case_types)]
pub struct READMEGenerator {}
impl READMEGenerator {
  /// Get the readme string contents
  fn get_readme_strings(&self, cli: &Cli) -> String{
    let lib_name = cli.get_lib_name();
    let generation_timestamp = cli.get_generation_timestamp_string();
    let InnerCli { site_or_api_name, api_url, api_spec_url_opt, .. } = &cli.inner_cli;
    let mut s = format!("
      # {lib_name}  
      
      This library:
      - Was *generated* at {generation_timestamp}.  
      - Implements the [{site_or_api_name}]({api_url}).  
      ");
    if let Some(api_spec_url) = api_spec_url_opt {
      s.push_str(&format!(
        "\n- Uses the corresponding OpenAPI specification found at [{api_spec_url}]." 
      ));
    }
    trim_lines(&s)
  }
  /// Instantiate 
  pub fn new(_cli: &Cli ) -> Result<Self, READMEGenerationError> { Ok(Self{}) }
  /// Write out to readme file 
  pub async fn write_to_readme_md_file(&self, cli: &Cli) -> Result<(), READMEGenerationError> {
    let readme_path = cli.get_output_project_subpath(&Paths::ReadmeMdFile);
    fs::write(&readme_path, self.get_readme_strings(cli)).await?;
    println!("Wrote README.md `{readme_path:?}`");
    Ok(())
  }
}