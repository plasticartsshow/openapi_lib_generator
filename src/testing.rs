//! Testing specifications
//!
//! These tests fall beyond the scope of cargo test because they are invoked from the CLI
use crate::generate::utils::ProcessError;
use std::io::Error as IOError;
use thiserror::Error;

/// A fake OpenAPI specification
pub static PETSTORE_YAML: &'static str = include_str!("testing/petstore.yaml");
/// A name for a testing OpenAPI yaml spec file
pub static TESTING_SPEC_FILE_NAME: &'static str = "petshoppe_test_spec.yaml";
/// A testing folder name
pub static TEST_SUBDIR_NAME: &'static str = "testing";
/// A testing api url
pub static TEST_API_URL: &'static str = "https://www.petshoppe.example";
/// A testing api name
pub static TEST_API_NAME: &'static str = "PetShoppe";
/// Testing errors
#[derive(Debug, Error)]
pub enum TestingError {
  #[error(transparent)]
  IOError(#[from] IOError),
  #[error(transparent)]
  ProcessError(#[from] ProcessError),
}
