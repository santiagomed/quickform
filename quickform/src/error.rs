use quickform_utils::template::TemplateEngineError;
use quickform_utils::fs::FSError;
#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("Template engine error")]
    TemplateEngineError(#[from] TemplateEngineError),
    #[error("In memory filesystem error")]
    FileSystemError(#[from] FSError),
    #[error("IO error")]
    IOError(#[from] std::io::Error),
}
