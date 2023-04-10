//! README file generation

use crate::{
  cli::{Cli, InnerCli, Paths},
  generate::utils::*,
};
use fs_err::tokio as fs;
use serde::{Deserialize, Serialize};
use std::io::Error as IOError;
use strum::EnumProperty;
use thiserror::Error;
/// Errors that can happen with yaml generation
#[derive(Debug, Error)]
pub enum READMEGenerationError {
  #[error(transparent)]
  IOError(#[from] IOError),
  // #[error(transparent)] SerdeYAMLError(#[from] SerdeYAMLError),
}

/// Readme generation
#[derive(Debug, Deserialize, Serialize)]
#[allow(non_camel_case_types)]
pub struct READMEGenerator {
  readme_string: String,
}
impl READMEGenerator {
  /// Get the readme string contents
  fn make_readme_string(cli: &Cli) -> String {
    let lib_name = cli.get_lib_name();
    let this_crate_name = get_this_crate_name().to_string();
    let this_crate_ver = get_this_crate_ver().to_string();
    let generation_timestamp = cli.get_generation_timestamp_string();
    let mut extra_authors_vec = cli.get_extra_authors();
    let eal = extra_authors_vec.len();
    let extra_authors =
      extra_authors_vec
        .drain(0..)
        .enumerate()
        .fold("".to_string(), |mut s, (i, c)| {
          if i == 0 {
            s.push_str("\nAdditional authors: ")
          }
          s.push_str(&format!("{c}"));
          if i < eal - 1 {
            s.push_str(", ");
          }
          s
        });
    let InnerCli {
      site_or_api_name,
      api_url,
      api_spec_url_opt,
      ..
    } = &cli.inner_cli;
    let mut s = format!("
      {extra_authors}

      ## About working on `{lib_name}`
      
      Hey! This library:
      - Was *generated* using {this_crate_name} v{this_crate_ver} at {generation_timestamp}. 
      - Implements the [{site_or_api_name}]({api_url}).
      
      
      For these reasons, proposed changes to this repository will likely not be accepted. Try proposing changes to the generator tools instead.

      ");
    if let Some(api_spec_url) = api_spec_url_opt {
      s.push_str(&format!(
        "\n- Uses the corresponding OpenAPI specification found at [{api_spec_url}]."
      ));
    }
    trim_lines(&s)
  }
  /// Instantiate
  pub fn new(cli: &Cli) -> Result<Self, READMEGenerationError> {
    let readme_string = Self::make_readme_string(cli);
    Ok(Self { readme_string })
  }
  /// Write out to readme file
  pub async fn update_readme_md_file(&self) -> Result<(), READMEGenerationError> {
    let readme_path = Paths::ReadmeMdFile
      .get_str("path")
      .expect("must get Cargo.toml path");
    fs::write(&readme_path, &self.readme_string).await?;
    println!("Wrote README.md `{readme_path:?}`");
    Ok(())
  }
}
