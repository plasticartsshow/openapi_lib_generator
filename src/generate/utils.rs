//! Codegen utilities

/// trim whitespace from multiline code resulting in a single string
pub fn trim_lines(s: &str) -> String {
  s.lines().map(|line| format!("{}\n", line.trim())).collect()
}
/// trim whitespace from multiline code resulting in a vec of strings
pub fn trim_lines_vec(s: &str) -> Vec<String> {
  s.lines().map(|line| format!("{}\n", line.trim())).collect()
}