//! Code generation
pub mod cargos;
pub mod crate_scaffolds;
pub mod makefiles;
pub mod parameters;
pub mod readmes;

pub mod utils;
// pub use utils::*;
pub mod yamls;

pub mod errors {
  use super::*;
  pub use cargos::CargoConfigError;
  pub use crate_scaffolds::CrateScaffoldingError;
  pub use makefiles::MakefileGenerationError;
  pub use parameters::ParameterError;
  pub use readmes::READMEGenerationError;
  pub use utils::ProcessError;
  pub use yamls::YAMLGenerationError;
}
pub use errors::*;
