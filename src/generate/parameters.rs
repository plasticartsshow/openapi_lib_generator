//! Generator errors
// use std::{path::{Path}};

use thiserror::Error; 
use url::Url;

/// Parameter errors
#[derive(Error, Debug, )]
pub enum ParameterError {
  #[error("API path has no segments {0}")] APIPathNeedsSegments(Url),    
  #[error("API path segments has no last")] APIPathSegmentsNeedsLast,    
}

/// Get file name from path
pub fn try_file_name_from_path_url(path_url: &Url) -> Result<String, ParameterError> {
  path_url
    .path_segments()
    .ok_or_else(|| ParameterError::APIPathNeedsSegments(path_url.clone()))
    .and_then(|path_segments| {
      path_segments.last()
        .ok_or_else(|| ParameterError::APIPathSegmentsNeedsLast)
        .map(ToString::to_string)
    })
}