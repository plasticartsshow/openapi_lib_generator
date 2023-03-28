//! Set up target crate

use crate::{cli::{Cli, Paths}};
use futures::{future::{TryFutureExt}};
use strum::EnumProperty;
use tokio::{fs, process};
use thiserror::Error;
use std::{
  io::{Error as IOError,}, 
  path::{PathBuf},
  process::{Output},
};

/// Crate scaffolding errors 
#[derive(Debug, Error)]
pub enum CrateScaffoldingError{ 
  #[error(transparent)] IOError(#[from] IOError),
  #[error("Could not find crate dir at {0}")] MissingCrateDir(PathBuf),
  #[error("Cargo init project at `{crate_dir}` failed with `{error_string}`  ")] CargoInitFailed{
    crate_dir: PathBuf, error_string: String
  },
}

/// Create the folder for the crate if it does not exist
async fn create_crate_folder(
  cli: &Cli,
) -> Result<(),  CrateScaffoldingError> {
  let dir_path = &cli.get_output_project_dir();
  fs::remove_dir_all(dir_path).await?;
  fs::create_dir_all(dir_path).await?;
  Ok(())
}

/// Initialize the crate
async fn init_crate(
  cli: &Cli
) -> Result<(), CrateScaffoldingError> {
  let dir_path = &cli.get_output_project_dir();
  async { Ok(dir_path.is_dir()) }
    .and_then(|is_dir| async move {
      if !is_dir {
        Err(CrateScaffoldingError::MissingCrateDir(dir_path.clone()))
      } else {
        let dir_path_string = dir_path.to_string_lossy().to_string();
        process::Command::new("cargo")
          .args(&[
            "init".to_string(), 
            "--lib".to_string(),
            "--color".to_string(),  "always".to_string(),
            dir_path_string.to_string(),
          ])
          .output()
          .await
          .map_err(CrateScaffoldingError::from)
          .and_then(|Output { 
            status, 
            stderr,
            stdout
          }| {
            if status.success() {
              let success_string = String::from_utf8(stdout)
                .unwrap_or_default();
              println!("Initialized crate at `{dir_path_string}` with output  {success_string}");
              Ok(())
            } else {
              let e = CrateScaffoldingError::CargoInitFailed { 
                crate_dir: dir_path.clone(), 
                error_string: String::from_utf8(stderr)
                  .unwrap_or_else(|_| "Error missing".to_string())
              };
              eprintln!("{e:?}");
              Err(e)
            }
          })
      }
    }).await
}

/// Do all crate scaffolding jobs
pub async fn scaffold_crate(cli: &Cli) -> Result<(), CrateScaffoldingError> {
  create_crate_folder(cli).await?;
  init_crate(cli).await?;
  setup_tree_in_crate(cli).await?;
  setup_git_in_crate(cli).await?;
  Ok(())
}


/// Setup file trees in crate
async fn setup_tree_in_crate(cli: &Cli) -> Result<(), CrateScaffoldingError> {
  // let crate_dir_path = cli.get_output_project_dir();
  let crate_temp_dir_path = cli.get_output_project_subpath(&Paths::TempDir);
  fs::create_dir_all(crate_temp_dir_path).await?;
  Ok(())
}


/// Setup git details in crate 
async fn setup_git_in_crate(cli: &Cli) -> Result<(), CrateScaffoldingError> {
  // let crate_dir_path = cli.get_output_project_dir();
  let crate_temp_dir_str = Paths::TempDir.get_str("path").expect("must get temp dir path");
  let gitignore_path = cli.get_output_project_subpath(&Paths::GitignoreFile);
  fs::write(&gitignore_path, &format!("\n/{crate_temp_dir_str}")).await?;
  Ok(())
}