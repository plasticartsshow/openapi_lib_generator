//! Code generation
pub mod crate_scaffolds;
pub mod parameters;
pub mod makefiles;
pub mod yamls;

pub mod errors {
  use super::*;
  pub use crate_scaffolds::CrateScaffoldingError;
  pub use parameters::ParameterError;
  pub use makefiles::MakefileGenerationError;
  pub use yamls::YAMLGenerationError;
}
pub use errors::*;