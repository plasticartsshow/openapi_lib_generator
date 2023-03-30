//! Codegen utilities
use std::env;



#[macro_export]
/// Just makes a vec of specified items from arguments
macro_rules! vv {
  (strings $($e:expr,)*) => {{vec![$($e.to_string(),)* ]}};
  (dep_names $($e:expr,)*) => {{vec![$(DependencyIdentifier::Name($e.to_string()),)* ]}};
} 


/// trim whitespace from multiline code resulting in a single string
pub fn trim_lines(s: &str) -> String {
  s.lines().map(|line| format!("{}\n", line.trim())).collect()
}
/// trim whitespace from multiline code resulting in a vec of strings
pub fn trim_lines_vec(s: &str) -> Vec<String> {
  s.lines().map(|line| format!("{}\n", line.trim())).collect()
}

/// Get the name of this crate 
pub fn get_this_crate_name() -> &'static str {
  env!("CARGO_CRATE_NAME")
}
/// Get the version of this crate 
pub fn get_this_crate_ver() -> &'static str {
  env!("CARGO_PKG_VERSION")
}