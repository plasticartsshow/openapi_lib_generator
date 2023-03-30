//! Cargo files generation 
use cargo_toml::{*, Error as CargoTomlError };
use crate::{cli::{Cli, InnerCli, Paths}, generate::{utils}, fs, vv,};
use serde::{Deserialize, Serialize};
use std::{
  io::{Error as IOError},
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
  /// The timestamp of generation
  pub generation_timestamp_string: String,
  /// The authors who ran the generator to add to the manifest authors
  pub generation_authors: Vec<String>,
  /// The name of this crate 
  pub this_crate_name: String,
  /// The version of this crate 
  pub this_crate_ver: String,
  /// The original api name
  pub original_api_name: String,
}
impl CargoConfigurator {
  /// Instantiate 
  pub fn new(cli: &Cli) -> Result<Self, CargoConfigError> {
    let mut generation_authors = InnerCli::parse_authors_string(env!("CARGO_PKG_AUTHORS"));
    generation_authors.extend(cli.get_extra_authors().drain(0..));
    let original_api_name = cli.inner_cli.site_or_api_name.to_string();
    let this_crate_name = utils::get_this_crate_name().to_string();
    let this_crate_ver = utils::get_this_crate_ver().to_string();
    let generation_timestamp_string = cli.get_generation_timestamp_string();
    Ok(Self {
      generation_timestamp_string,
      generation_authors,
      this_crate_name,
      this_crate_ver,
      original_api_name,
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
    p.authors
      .get_mut()?
      .extend(self.generation_authors.iter().cloned());
    p.description.get_or_insert_with(Default::default)
      .get_mut()?
      .push_str(&format!("\n Generated at {}", self.generation_timestamp_string));
    p.keywords
      .get_mut()?
      .extend(vv![strings self.original_api_name.as_str(), "OpenAPI", "web",].into_iter());
    p.categories_mut()
      .extend(vv![strings "web-programming", "api-bindings", "authentication", ]);
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

