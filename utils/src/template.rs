// In a separate helper crate (e.g., template-helper/src/lib.rs)
use serde::Serialize;
use std::path::Path;
use tera::Tera;

#[derive(thiserror::Error, Debug)]
pub(crate) enum TemplateEngineError {
    #[error("Invalid template path")]
    InvalidTemplatePath,
    #[error("Failed to render template")]
    RenderError(#[from] tera::Error),
}

pub struct TemplateEngine {
    tera: Tera,
}

impl TemplateEngine {
    /// Creates a new empty template engine instance without a template directory
    pub fn new() -> Self {
        Self { 
            tera: Tera::default() 
        }
    }

    /// Creates a new template engine instance from a directory path
    pub fn with_dir<P: AsRef<Path>>(template_dir: P) -> Result<Self, TemplateEngineError> {
        let template_path = template_dir.as_ref().join("**").join("*.html");
        let template_path_str = template_path.to_str()
            .ok_or(TemplateEngineError::InvalidTemplatePath)?;
            
        let tera = Tera::new(template_path_str)?;
        
        Ok(Self { tera })
    }

    /// Renders a template with the given context
    pub fn render<T: Serialize>(&self, template_name: &str, context: &T) -> Result<String, TemplateEngineError> {
        let rendered = self.tera.render(template_name, &tera::Context::from_serialize(context)?)?;
        Ok(rendered)
    }

    /// Adds or updates a template from a string
    pub fn add_template_string(&mut self, name: &str, content: &str) -> Result<(), TemplateEngineError> {
        self.tera.add_raw_template(name, content)?;
        Ok(())
    }

    /// Reloads all templates from the directory
    pub fn reload(&mut self) -> Result<(), TemplateEngineError> {
        self.tera.full_reload()?;
        Ok(())
    }

    /// Returns a reference to the internal Tera instance
    pub fn get_tera(&self) -> &Tera {
        &self.tera
    }
}

