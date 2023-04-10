//! Codegen utilities
use crate::{
  testing,
  cli::{Cli},
};
use std::{ 
  env,
  path::{Path, PathBuf},
  io::{Error as IOError},
  process::{Output}
};
use thiserror::Error;
use tokio::{process::{Child, Command}};

/// Testing errors 
#[derive(Debug, Error)]
pub enum ProcessError {
  #[error(transparent)] IOError(#[from] IOError),
  #[error("Process failed \n {0}")] Failure(String),
}


#[macro_export]
/// Just makes a vec of specified items from arguments
macro_rules! vv {
  (strings $($e:expr,)*) => {{vec![$($e.to_string(),)* ]}};
  (dep_names $($e:expr,)*) => {{vec![$(DependencyIdentifier::Name($e.to_string()),)* ]}};
  (as_ref dep_names $($e:expr,)*) => {{ $crate::vv![dep_names $($e.as_ref(),)* ]}};
} 


/// Attempt to run a cargo job
pub async fn run_cargo_job<T: AsRef<str>, P: AsRef<Path>>(
  args: &[T], 
  cwd_opt: Option<P>,
  description_opt: Option<T>,
) -> Result<Output, ProcessError> {
  let mut command = Command::new("cargo");
  let cwd_string = if let Some(cwd) = cwd_opt.as_ref() {
    command.current_dir(&cwd.as_ref());
    format!("in {}", cwd.as_ref().to_string_lossy())
  } else { String::default() };
  let args_vec = &Vec::from_iter(args.iter().map(AsRef::as_ref));
  let arg_string = args_vec.iter().map(|a| format!("\"{a}\" ")).collect::<String>();
  let description = description_opt.map(|s| s.as_ref().to_string())
    .unwrap_or_else(|| format!("Running `cargo` {arg_string} {cwd_string}.", ));
  println!("{description}");
  let child: Child = command
    .args(args_vec)
    .spawn()?;
  child
    .wait_with_output().await
    .map_err(ProcessError::from)
}


/// Attempt to run a cargo make task
pub async fn run_cargo_make_task<T: AsRef<str>>(cli: &Cli, task_name: T) -> Result<Output, ProcessError> {
  let output_project_dir = &cli.get_output_project_dir();
  run_cargo_job(
    &["make", task_name.as_ref()], 
    Some(output_project_dir),
    None,
  ).await
}

/// trim leading whitespace from multiline code resulting in a single string
pub fn trim_lines(s: &str) -> String { trim_lines_vec(s).join("/n") }
/// trim leading whitespace from multiline code resulting in a vec of strings
pub fn trim_lines_vec(s: &str) -> Vec<String> {
  s.lines()
    .map(|line| line.trim_end())
    .filter(|line| !line.is_empty())
    .fold(
      ( 
        String::new(),
        Vec::new()
      ),
      | 
        (mut target, mut result), line
      | {
        if target.is_empty() && !line.is_empty() {
          let leading_whitespace = line.len() - line.trim_start_matches(" ").len();
          target = " ".repeat(leading_whitespace);
        }
        if !target.is_empty() {
          result.push(format!("{}", line.strip_prefix(&target).unwrap_or(line)));
        }
        (target, result)
      }
    ).1
}

/// Get the name of this crate 
pub fn get_this_crate_name() -> &'static str {
  env!("CARGO_CRATE_NAME")
}
/// Get the version of this crate 
pub fn get_this_crate_ver() -> &'static str {
  env!("CARGO_PKG_VERSION")
}
/// Get the version of this crate with a 'v'
pub fn get_this_crate_ver_pretty() -> String { format!("v{}", get_this_crate_ver())
}
/// Get the temp root directory 
pub fn get_temp_root_dir() -> PathBuf { env::temp_dir() }
/// Get temp project subdir 
pub fn get_temp_subdir() -> PathBuf { 
  get_temp_root_dir().join(&format!("{}_{}", get_this_crate_name(), testing::TEST_SUBDIR_NAME))
}

#[cfg(test)]
mod test_mod_name {
  use super::*;
  #[test]
  fn trim_lines_works() {
    let test = r#"
        line zero
          line one
    "#;
    assert_eq!(
      trim_lines_vec(test),
      vec!["line zero", "  line one"]
    )
  }
}