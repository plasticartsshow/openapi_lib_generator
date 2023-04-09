//! Open api lib generator CLI

use openapi_lib_generator::{
  cli::*,
  generate::{
    crate_scaffolds,
    makefiles::{MakefileSpec, TaskNames},
    yamls::{OpenAPIRustGeneratorConfigs}, CrateScaffoldingError,
    utils::{ProcessError, run_cargo_make_task}
  },
  testing::{TestingError}
};

/// Run a subcommand 
async fn run_subcommands(cli: &Cli) -> Result<(), CLIError> {
  let Cli {
    inner_cli: InnerCli { 
      api_spec_url_opt, 
      autogenerate,
      .. 
    },
    ..
  } = cli;
  match cli.command.as_ref() {
    Some(SubCommands::TestGeneration { .. }) => {
      let task_name = TaskNames::GenerateAll;
      let output = run_cargo_make_task(cli, task_name).await
        .map_err(TestingError::from)
        .map_err(CLIError::from)?;
      if !output.status.success() {
        Err(CLIError::from(
          TestingError::ProcessError( ProcessError::Failure(format!("{output:#?}")))
        ))
      } else {
        Ok(())
      }
    },
    None => {
      if *autogenerate && api_spec_url_opt.is_some() {
        let task_name = TaskNames::SpecDownloadDefault;
        let output = run_cargo_make_task(cli, task_name).await
        .map_err(CrateScaffoldingError::from)
        .map_err(CLIError::from)?;
        if !output.status.success() {
          Err(CLIError::from(ProcessError::Failure(format!("{output:#?}"))))
        } else {
          let task_name = TaskNames::GenerateAll;
          let output = run_cargo_make_task(cli, task_name).await
          .map_err(CrateScaffoldingError::from)
          .map_err(CLIError::from)?;
          if !output.status.success() {
            Err(CLIError::from(ProcessError::Failure(format!("{output:#?}"))))
          } else {
            Ok(())
          }
        }
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
  run_subcommands(cli).await?;
  
  Ok(())
}