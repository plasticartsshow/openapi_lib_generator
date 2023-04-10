//! Cargo files generation
use crate::{
  cli::{Cli, InnerCli, Paths, SubCommands},
  fs,
  generate::utils,
  vv,
};
use cargo_toml::{Edition, Error as CargoTomlError, Product, *};
use serde::{Deserialize, Serialize};
use std::io::Error as IOError;
use strum::EnumProperty;
use thiserror::Error;
use toml::ser::Error as TomlSerError;

/// Errors
#[derive(Error, Debug)]
pub enum CargoConfigError {
  #[error(transparent)]
  CargoTomlError(#[from] CargoTomlError),
  #[error(transparent)]
  TomlSerError(#[from] TomlSerError),
  #[error("Updating from the Rust edition '{0:?}' is currently unsupported")]
  UpdateRustEditionError(Edition),
  #[error(transparent)]
  IOError(#[from] IOError),
}

/// Cargo toml modifier
#[derive(Debug, Deserialize, Serialize)]
pub struct CargoConfigurator {
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
  /// The cli subcommand if applicable
  pub subcommand_opt: Option<SubCommands>,
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
    let subcommand_opt = cli.inner_cli.command.clone();
    // dbg!(&subcommand_opt);
    Ok(Self {
      generation_timestamp_string,
      generation_authors,
      this_crate_name,
      this_crate_ver,
      original_api_name,
      subcommand_opt,
    })
  }

  /// Update the given manifest edition value if possible
  fn update_edition(
    description: &str,
    edition: &mut Edition,
  ) -> Result<String, CargoConfigError> {
    let make_report = |e0, e1| Ok(format!("Updated {description} from {e0:?} to {e1:?}"));
    match *edition {
      e0 @ Edition::E2015 => {
        *edition = Edition::E2018;
        make_report(e0, *edition)
      }
      e0 @ Edition::E2018 => {
        *edition = Edition::E2021;
        make_report(e0, *edition)
      }
      // e0 @ Edition::E2021 => {
      //   *edition = Edition::E2024;
      //   make_report(e0, *edition)
      // },
      // e0 @ Edition::E2024 => {
      //   *edition = Edition::E2027;
      //   make_report(e0, *edition)
      // },
      edition => Err(CargoConfigError::UpdateRustEditionError(edition)),
    }
  }

  /// Update a cargo.toml file **AFTER** cargo check --edition
  pub async fn update_cargo_manifest_post_fix_edition(&self) -> Result<(), CargoConfigError> {
    let cargo_toml_path = Paths::CargoTomlFile
      .get_str("path")
      .expect("must get Cargo.toml path");
    let cargo_manifest = &mut Manifest::<String>::from_path_with_metadata(cargo_toml_path)?;
    let Manifest { package, lib, .. } = cargo_manifest;
    let p = package.get_or_insert_with(Default::default);
    let package_update_desc = Self::update_edition("manifest package", p.edition.get_mut()?)?;
    let lib_update_desc = lib
      .as_mut()
      .map(|Product { edition, .. }| Self::update_edition("manifest lib target", edition))
      .unwrap_or_else(|| Ok(Default::default()))?;
    fs::write(
      cargo_toml_path,
      toml::to_string_pretty(cargo_manifest)?,
      Some(&format!(
        "updated cargo manifest edition post fix ({package_update_desc}, {lib_update_desc})"
      )),
    )
    .await?;
    Ok(())
  }

  /// Update a cargo.toml file **AFTER** code generation
  pub async fn update_cargo_manifest_post_generation(&self) -> Result<(), CargoConfigError> {
    // dbg!(self);
    let cargo_toml_path = Paths::CargoTomlFile
      .get_str("path")
      .expect("must get Cargo.toml path");
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
    p.description
      .get_or_insert_with(Default::default)
      .get_mut()?
      .push_str(&format!(
        "\n Generated at {}",
        self.generation_timestamp_string
      ));
    p.keywords
      .get_mut()?
      .extend(vv![strings self.original_api_name.as_str(), "OpenAPI", "web",].into_iter());
    p.categories_mut()
      .extend(vv![strings "web-programming", "api-bindings", "authentication", ]);
    let mut this_crate_dependency: Dependency = Dependency::Detailed(DependencyDetail {
      version: Some(self.this_crate_ver.to_string()),
      ..Default::default()
    });
    if let Some(subcommand) = self.subcommand_opt.as_ref() {
      match subcommand {
        SubCommands::TestGeneration {
          generator_crate_local_path_opt,
          generator_crate_repo_url_opt,
        } => match generator_crate_local_path_opt {
          Some(generator_crate_local_path) => {
            this_crate_dependency
              .detail_mut()
              .path
              .replace(generator_crate_local_path.to_string_lossy().to_string());
          }
          None => match generator_crate_repo_url_opt {
            Some(generator_crate_repo_url) => {
              this_crate_dependency
                .detail_mut()
                .git
                .replace(generator_crate_repo_url.as_str().to_string());
            }
            None => {}
          },
        },
      }
    }
    dev_dependencies.insert(self.this_crate_name.to_string(), this_crate_dependency);
    fs::write(
      cargo_toml_path,
      toml::to_string_pretty(cargo_manifest)?,
      Some("updated cargo manifest post generation"),
    )
    .await?;
    Ok(())
  }
}
