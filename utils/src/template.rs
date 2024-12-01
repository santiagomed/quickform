// In a separate helper crate (e.g., template-helper/src/lib.rs)
use serde::Serialize;
use std::path::Path;
use minijinja::Environment;

#[derive(thiserror::Error, Debug)]
pub enum TemplateEngineError {
    #[error("Invalid template path")]
    InvalidTemplatePath,
    #[error("Failed to render template")]
    RenderError(#[from] minijinja::Error),
}

pub struct TemplateEngine<'a> {
    env: Environment<'a>,
}

impl<'a> TemplateEngine<'a> {
    /// Creates a new empty template engine instance without a template directory
    pub fn new() -> Self {
        Self { 
            env: Environment::new() 
        }
    }

    /// Creates a new template engine instance from a directory path
    pub fn with_dir<P: AsRef<Path>>(template_dir: P) -> Result<Self, TemplateEngineError> {
        let mut env = Environment::new();
        env.set_loader(minijinja::path_loader(template_dir.as_ref()));
        
        Ok(Self { env })
    }

    /// Renders a template with the given context
    pub fn render<T: Serialize>(&self, template_name: &str, context: &T) -> Result<String, TemplateEngineError> {
        let tmpl = self.env.get_template(template_name)?;
        let rendered = tmpl.render(context)?;
        Ok(rendered)
    }

    /// Adds or updates a template from a string
    pub fn add_template_string(&mut self, name: &'a str, content: &'a str) -> Result<(), TemplateEngineError> {
        self.env.add_template(name, content)?;
        Ok(())
    }

    /// Returns a reference to the internal Environment instance
    pub fn get_environment(&self) -> &Environment {
        &self.env
    }
}

