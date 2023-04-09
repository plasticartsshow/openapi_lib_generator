//! Testing specifications 
//! 
//! These tests fall beyond the scope of cargo test because they are invoked from the CLI
use thiserror::Error;
use std::{
  io::Error as IOError,
};

/// A fake OpenAPI specification
pub static PETSTORE_YAML : &'static str = include_str!("testing/petstore.yaml"); 
/// A name for a testing OpenAPI yaml spec file
pub static TESTING_SPEC_FILE_NAME : &'static str = "petshoppe_test_spec.yaml";
/// A testing folder name 
pub static TEST_SUBDIR_NAME: &'static str = "testing";
/// Testing errors 
#[derive(Debug, Error)]
pub enum TestingError {
  #[error(transparent)] IOError(#[from]IOError),
  #[error("Test process failed \n {0}")] TestProcessFailure(String),
}

