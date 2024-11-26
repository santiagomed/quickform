mod node;
pub mod template;
use crate::fs::FileSystem;

// Represents the shared context between nodes
#[derive(Default, Debug)]
pub struct GenerationContext {
    // User prompt
    pub user_prompt: String,
    // Store generated files and their contents
    pub fs: FileSystem,
    // Store starter requirements information
    pub entities: Vec<schemas::entity::Entity>,
}

impl GenerationContext {
    pub fn new(user_prompt: String) -> Self {
        Self {
            user_prompt,
            ..Default::default()
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum GenerationError {
    #[error("Template error: {0}")]
    TemplateError(#[from] template::TemplateError),
    #[error("Schema error: {0}")]
    SchemaError(#[from] serde_json::Error),
    #[error("Filesystem error: {0}")]
    FileSystemError(String),
    #[error("Context error: {0}")]
    ContextError(String),
}

pub struct Generator {
    operations: Vec<
        Box<
            dyn Fn(
                    &mut GenerationContext,
                ) -> futures::future::BoxFuture<Result<String, GenerationError>>
                + Send
                + Sync,
        >,
    >,
    context: GenerationContext,
}

impl Generator {
    pub fn new(prompt: String) -> Self {
        Self {
            operations: Vec::new(),
            context: GenerationContext::new(prompt),
        }
    }

    pub fn operation<F, Fut>(&mut self, operation: F) -> &mut Self
    where
        F: Fn(&mut GenerationContext) -> Fut + Send + Sync + 'static,
        Fut: futures::Future<Output = Result<String, GenerationError>> + Send + 'static,
    {
        self.operations
            .push(Box::new(move |ctx| Box::pin(operation(ctx))));
        self
    }

    pub async fn run(&mut self) -> Result<(), GenerationError> {
        for operation in &self.operations {
            operation(&mut self.context).await?;
        }
        Ok(())
    }
}
