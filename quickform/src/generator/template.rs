// In a separate helper crate (e.g., template-helper/src/lib.rs)
use serde::Serialize;
use std::path::Path;
use tera::Tera;

#[derive(Debug)]
pub struct TemplateError(String);

impl std::error::Error for TemplateError {}
impl std::fmt::Display for TemplateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Template error: {}", self.0)
    }
}

pub fn render_template<T>(
    template_path: &str,
    data: T,
) -> Result<String, Box<dyn std::error::Error>>
where
    T: Serialize,
{
    let path = Path::new(template_path);
    let mut tera = Tera::default();

    tera.add_template_file(path, Some("template"))
        .map_err(|e| TemplateError(format!("Failed to load template: {}", e)))?;

    let context = tera::Context::from_serialize(data)
        .map_err(|e| TemplateError(format!("Failed to create context: {}", e)))?;

    tera.render("template", &context)
        .map_err(|e| TemplateError(format!("Failed to render template: {}", e)).into())
}
