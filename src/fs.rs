//! Filesystem operations
use fs_err::tokio as fs;
use std::{io::Error as IOError, path::Path};

/// Write to file with logging
pub async fn write(
  path: impl AsRef<Path>,
  contents: impl AsRef<[u8]>,
  message: Option<impl AsRef<str>>,
) -> Result<(), IOError> {
  let bytes = contents.as_ref().len();
  let pp = path.as_ref().as_os_str();
  fs::write(&path, contents).await?;
  let wrote_message = format!(" Wrote {bytes} bytes to {pp:?}");
  match message {
    Some(s) => println!("{}: {wrote_message}", s.as_ref()),
    None => println!("{wrote_message}"),
  }
  Ok(())
}
