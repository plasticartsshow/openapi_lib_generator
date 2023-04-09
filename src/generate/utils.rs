//! Codegen utilities
use crate::{
  testing
};
use std::{ 
  env,
  path::{PathBuf}
};



#[macro_export]
/// Just makes a vec of specified items from arguments
macro_rules! vv {
  (strings $($e:expr,)*) => {{vec![$($e.to_string(),)* ]}};
  (dep_names $($e:expr,)*) => {{vec![$(DependencyIdentifier::Name($e.to_string()),)* ]}};
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