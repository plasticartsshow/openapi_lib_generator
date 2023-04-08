//! Open api lib generator CLI

use openapi_lib_generator::{
  cli::*,
  generate::{
    crate_scaffolds,
    makefiles::{MakefileSpec, TaskNames},
    yamls::{OpenAPIRustGeneratorConfigs},
  },
  testing::{TestingError}
};

/// Run a subcommand in some generated code crate
async fn run_subcommand(subcommand: &SubCommands) -> Result<(), CLIError> {
  use tokio::{process::Command};
  match subcommand {
    SubCommands::TestGeneration { 
      // generator_crate_local_path_opt, 
      // generator_crate_repo_url_opt,
      ..
    } => {
      let task_name_str: &'static str = TaskNames::GenerateAll.as_ref();
      let output = Command::new("cargo")
        .args(&["make", task_name_str])
        .output().await
        .map_err(TestingError::from)
        .map_err(CLIError::from)?;
      if !output.status.success() {
        Err(CLIError::from(
          TestingError::TestProcessFailure(
            String::from_utf8_lossy(&output.stderr).to_string()
          )
        ))
      } else {
        
        Ok(())
      }

    }
  }
}

#[tokio::main]
async fn main() -> Result<(), CLIError> {
  let cli = &Cli::new().await?;
  crate_scaffolds::scaffold_crate(cli).await?;
  let makefile_spec = MakefileSpec::try_from(cli)?;
  makefile_spec.write_to_makefile(cli).await?;
  let rust_generator_configs = OpenAPIRustGeneratorConfigs::new(cli);
  rust_generator_configs.copy_spec_file(cli).await?;
  rust_generator_configs.write_to_yaml_file(cli).await?;
  if let Some(subcommand) = &cli.command {
    run_subcommand(subcommand).await?;
  } 
  Ok(())
}