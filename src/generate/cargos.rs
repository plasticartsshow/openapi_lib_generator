//! Cargo files generation 
use cargo_toml::{*, Error as CargoTomlError };
use crate::{cli::{Cli, InnerCli, Paths}, fs};
use serde::{Deserialize, Serialize};
use std::{
  env, io::{Error as IOError},
};
use strum::EnumProperty;
use thiserror::Error;
use toml::{ser::Error as TomlSerError};

/// Errors
#[derive(Error, Debug, )]
pub enum CargoConfigError {
  #[error(transparent)] CargoTomlError(#[from]CargoTomlError),
  #[error(transparent)] TomlSerError(#[from]TomlSerError),
  #[error(transparent)] IOError(#[from]IOError),
}


/// Cargo toml modifier 
#[derive(Deserialize, Serialize)]
pub struct CargoConfigurator{
  /// The authors who ran the generator to add to the manifest authors
  pub generation_authors: Vec<String>,
  /// The name of this crate 
  pub this_crate_name: String,
  /// The version of this crate 
  pub this_crate_ver: String,
}
impl CargoConfigurator {
  /// Instantiate 
  pub fn new(cli: &Cli) -> Result<Self, CargoConfigError> {
    let mut generation_authors = InnerCli::parse_authors_string(env!("CARGO_PKG_AUTHORS"));
    generation_authors.extend(cli.get_extra_authors().drain(0..));
    let this_crate_name = env!("CARGO_CRATE_NAME").to_string();
    let this_crate_ver = env!("CARGO_PKG_VERSION").to_string();
    Ok(Self {
      generation_authors,
      this_crate_name,
      this_crate_ver,
    })
  }
  /// Update a cargo.toml file **AFTER** code generation
  pub async fn update_cargo_toml(&self) -> Result<(), CargoConfigError> {
    let cargo_toml_path = Paths::CargoTomlFile.get_str("path").expect("must get Cargo.toml path");
    let cargo_manifest = &mut Manifest::<String>::from_path_with_metadata(cargo_toml_path)?;
    let Manifest {
      package, 
      dev_dependencies,
      ..
    } = cargo_manifest;
    let p = package.get_or_insert_with(Default::default);
    p.authors.get_mut()?
      .extend(self.generation_authors.iter().cloned());
    dev_dependencies.insert(
      self.this_crate_name.to_string(), 
      Dependency::Detailed(
        DependencyDetail { 
          version: Some(self.this_crate_ver.to_string()),
          ..Default::default()
        }
      )
    );
    fs::write(
      cargo_toml_path, 
      toml::to_string_pretty(cargo_manifest)?,
      Some("updated cargo.toml")
    ).await?;    
    Ok(())
  }
}

