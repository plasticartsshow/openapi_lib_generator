//! Code generation
pub mod crate_scaffolds;
pub mod cargos;
pub mod parameters;
pub mod makefiles;
pub mod readmes;


pub mod utils;
pub use utils::*;
pub mod yamls;


pub mod errors {
  use super::*;
  pub use crate_scaffolds::CrateScaffoldingError;
  pub use cargos::CargoConfigError;
  pub use parameters::ParameterError;
  pub use makefiles::MakefileGenerationError;
  pub use readmes::READMEGenerationError;
  pub use yamls::YAMLGenerationError;
}
pub use errors::*;