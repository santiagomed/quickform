// In a separate helper crate (e.g., template-helper/src/lib.rs)
use serde::Serialize;
use std::path::Path;
use minijinja::Environment;

pub(crate) struct TemplateEngine<'a> {
    env: Environment<'a>,
}

impl<'a> TemplateEngine<'a> {
    /// Creates a new empty template engine instance without a template directory
    pub(crate) fn new() -> Self {
        Self { 
            env: Environment::new() 
        }
    }

    /// Creates a new template engine instance from a directory path
    pub(crate) fn with_dir<P: AsRef<Path>>(template_dir: P) -> Self {
        let mut env = Environment::new();
        env.set_loader(minijinja::path_loader(template_dir.as_ref()));
        
        Self { env }
    }

    /// Renders a template with the given context
    pub(crate) fn render<T: Serialize>(&self, template_name: &str, context: &T) -> Result<String, minijinja::Error> {
        let tmpl = self.env.get_template(template_name)?;
        tmpl.render(context)
    }
}

