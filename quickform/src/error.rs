use quickform_utils::template::TemplateEngineError;
use quickform_utils::fs::FSError;

/// Represents all possible errors that can occur in the quickform library
///
/// This enum consolidates errors from various subsystems:
/// - Template rendering errors from the template engine
/// - File system operations errors
/// - Standard IO errors
///
/// # Examples
///
/// ```rust
/// use quickform::Error;
///
/// fn example_operation() -> Result<(), Error> {
///     // Operations that might fail can return this error type
///     // It will automatically convert from underlying error types
///     Ok(())
/// }
/// ```
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// An error occurred while processing templates
    #[error("Template engine error")]
    TemplateEngineError(#[from] TemplateEngineError),
    /// An error occurred during file system operations
    #[error("In memory filesystem error")]
    FileSystemError(#[from] FSError),
    /// An error occurred during IO operations
    #[error("IO error")]
    IOError(#[from] std::io::Error),
}
