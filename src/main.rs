//! Open api lib generator CLI

use openapi_lib_generator::{
  cli::*,
  generate::{
    crate_scaffolds,
    makefiles::{MakefileSpec},
    yamls::{OpenAPIRustGeneratorConfigs},
  }
};

#[tokio::main]
async fn main() -> Result<(), CLIError> {
  let cli = &Cli::new()?;
  crate_scaffolds::scaffold_crate(cli).await?;
  let makefile_spec = MakefileSpec::try_from(cli)?;
  makefile_spec.write_to_makefile(cli).await?;
  let rust_generator_configs = OpenAPIRustGeneratorConfigs::new(cli);
  rust_generator_configs.copy_spec_file(cli).await?;
  rust_generator_configs.write_to_yaml_file(cli).await?;
  Ok(())
}