//! Open api lib generator CLI

use openapi_lib_generator::{
  cli::*,
  generate::{
    crate_scaffolds,
    makefiles::{MakefileSpec},
    readmes::{READMEGenerator},
    yamls::{OpenAPIRustGeneratorConfigs},
  }
};

/// 

#[tokio::main]
async fn main() -> Result<(), CLIError> {
  let cli = &Cli::new()?;
  // let output_project_dir = cli.get_output_project_dir();
  crate_scaffolds::scaffold_crate(cli).await?;
  let rust_generator_configs = OpenAPIRustGeneratorConfigs::new(cli);
  rust_generator_configs.copy_spec_file(cli).await?;
  rust_generator_configs.write_to_yaml_file(cli).await?;
  let makefile_spec = MakefileSpec::try_from(cli)?;
  makefile_spec.write_to_makefile(cli).await?;
  let readme_generator = READMEGenerator::new(cli)?;
  readme_generator.write_to_readme_md_file(cli).await?;
  Ok(())
}