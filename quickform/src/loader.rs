use crate::fs::{FSError, MemFS};
use minijinja::Error;
use std::str;

/// Creates a template loader that loads templates from the MemFS.
pub fn memfs_loader(fs: &'static MemFS) -> impl Fn(&str) -> Result<Option<String>, Error> {
    move |name| {
        match fs.read_file(name) {
            Ok(content) => {
                // Convert bytes to string
                match str::from_utf8(content) {
                    Ok(s) => Ok(Some(s.to_string())),
                    Err(_) => Err(Error::new(
                        minijinja::ErrorKind::InvalidOperation,
                        "Template file contains invalid UTF-8",
                    )),
                }
            }
            Err(FSError::NotFound(_)) => Ok(None),
            Err(e) => Err(Error::new(
                minijinja::ErrorKind::InvalidOperation,
                format!("Failed to load template: {}", e),
            )),
        }
    }
}
