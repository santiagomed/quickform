// src/lib.rs
mod operation;
mod state;
mod context;
mod error;

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::path::Path;
use quickform_utils::fs::MemFS;
use quickform_utils::template::TemplateEngine;
use operation::{FunctionSignature, Operation};
use state::{Data, NoData, IntoFunctionParams};
use context::Context;
use error::AppError;

type BoxedOperation = Box<dyn Fn() -> Pin<Box<dyn Future<Output = Box<dyn Context>> + Send>> + Send + Sync>;

pub struct App<T> {
    state: T,
    operations: HashMap<String, BoxedOperation>,
    paths: Vec<String>,
    fs: MemFS,
    engine: TemplateEngine<'static>,
}

impl App<NoData> {
    pub fn new() -> Self {
        Self {
            state: NoData,  
            operations: HashMap::new(),
            paths: Vec::new(),
            fs: MemFS::new(),
            engine: TemplateEngine::new(),
        }
    }

    pub fn with_templates<P: AsRef<Path>>(self, template_dir: P) -> Result<Self, AppError> {
        let mut engine = TemplateEngine::with_dir(template_dir)?;
        Ok(Self {
            state: NoData,
            operations: self.operations,
            paths: self.paths,
            fs: self.fs,
            engine,
        })
    }

    pub fn with_state<S>(self, state: S) -> App<Data<S>> {
        App {
            state: Data::new(state),
            operations: self.operations,
            paths: self.paths,
            fs: self.fs,
            engine: self.engine,
        }
    }
}

// Base implementation for App with one data type
impl<S1: Send + Sync + 'static> App<Data<S1>> {
    pub fn with_state<S2>(self, state: S2) -> App<(Data<S1>, Data<S2>)> {
        App {
            state: (self.state, Data::new(state)),
            operations: self.operations,
            paths: self.paths,
            fs: self.fs,
            engine: self.engine,
        }
    }
}

// Or we could use a macro to generate these implementations:
macro_rules! impl_app_with_state {
    (($($idx:tt),*); $($prev:ident),*; $next:ident) => {
        impl<$($prev: Send + Sync + 'static,)*> App<($(Data<$prev>,)*)> {
            pub fn with_state<$next>(self, state: $next) -> App<($(Data<$prev>,)* Data<$next>)> {
                App {
                    state: ($(self.state.$idx,)* Data::new(state)),
                    operations: self.operations,
                    paths: self.paths,
                    fs: self.fs,
                    engine: self.engine,
                }
            }
        }
    };
}

// Generate implementations for tuples of different sizes
impl_app_with_state!((0); S1; S2);
impl_app_with_state!((0, 1); S1, S2; S3);
impl_app_with_state!((0, 1, 2); S1, S2, S3; S4);

impl<T: Send + Sync + Clone + 'static> App<T> {
    pub fn operation<FSig, F>(mut self, name: &str, operation: F) -> Self 
    where
        FSig: FunctionSignature + 'static,
        F: Operation<FSig> + Copy + Send + Sync + 'static,
        F::Future: Send + 'static,
        FSig::Output: Context,
        T: IntoFunctionParams<FSig>,
    {
        let state = self.state.clone();
        let wrapped_op = move || {
            let params = state.clone().into_params();
            let fut = operation.execute(params);
            Box::pin(async move {
                let result = fut.await;
                Box::new(result) as Box<dyn Context>
            }) as Pin<Box<dyn Future<Output = _> + Send>>
        };

        self.operations.insert(name.to_string(), Box::new(wrapped_op));
        self.paths.push(name.to_string());
        self
    }

    pub async fn run(&self) -> Result<(), AppError> {
        for path in &self.paths {
            self.render(path).await?;
        }
        Ok(())
    }
    
    async fn render(&self, name: &str) -> Result<String, AppError> {
        if let Some(op) = self.operations.get(name) {
            let context = op().await.to_value();
            let template_src = std::fs::read_to_string(name)?;
            Ok(minijinja::render!(&template_src, context))
        } else {
            Err(AppError::IOError(std::io::Error::new(std::io::ErrorKind::NotFound, format!("Operation not found: {}", name))))
        }
    }
}

// Test implementation
#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[derive(Clone, serde::Serialize)]
    struct User {
        name: String,
        age: u32,
    }

    #[derive(Clone, serde::Serialize)]
    struct Config {
        timeout: Duration,
    }

    #[tokio::test]
    async fn test_no_params() {
        async fn get_default_name() -> String {
            "Default".to_string()
        }

        let tmp_dir = tempdir::TempDir::new("test").unwrap();
        let template_path = tmp_dir.path().join("get_default.jinja");
        std::fs::write(&template_path, "{{ context }}").unwrap();
        let full_path = template_path.to_str().unwrap();

        let app = App::new()
            .operation(full_path, get_default_name);

        let result = app.render(full_path).await.unwrap();
        assert_eq!(result, "Default");
    }

    #[tokio::test]
    async fn test_single_param() {
        async fn double_age(user: Data<User>) -> User {
            User {
                name: user.name.clone(),
                age: user.age * 2,
            }
        }

        let tmp_dir = tempdir::TempDir::new("test").unwrap();
        let template_path = tmp_dir.path().join("double_age.jinja");
        std::fs::write(&template_path, "Age: {{ context.age }}").unwrap();
        let full_path = template_path.to_str().unwrap();

        let app = App::new()
            .with_state(User {
                name: "Alice".to_string(),
                age: 30,
            })
            .operation(full_path, double_age);

        app.run().await.unwrap();
    }

    #[tokio::test]
    async fn test_two_params() {
        async fn get_user_with_timeout(
            user: Data<User>,
            config: Data<Config>
        ) -> (String, Duration) {
            (user.name.clone(), config.timeout)
        }

        let tmp_dir = tempdir::TempDir::new("test").unwrap();
        let template_path = tmp_dir.path().join("get_user_timeout.jinja");
        std::fs::write(&template_path, "{{ context[0] }}").unwrap();
        let full_path = template_path.to_str().unwrap();

        let app = App::new()
            .with_state(User {
                name: "Bob".to_string(),
                age: 25,
            })
            .with_state(Config {
                timeout: Duration::from_secs(30),
            })
            .operation(full_path, get_user_with_timeout);

        let result = app.render(full_path).await.unwrap();
        assert_eq!(result, "Bob");
    }

    #[tokio::test]
    async fn test_three_params() {
        async fn three_params(x: Data<i32>, y: Data<i32>, z: Data<i32>) -> i32 {
            x.get_ref() + y.get_ref() + z.get_ref()
        }

        let tmp_dir = tempdir::TempDir::new("test").unwrap();
        let template_path = tmp_dir.path().join("three_params.jinja");
        std::fs::write(&template_path, "{{ context }}").unwrap();
        let full_path = template_path.to_str().unwrap();

        let app = App::new()
            .with_state(1)
            .with_state(2)
            .with_state(3)
            .operation(full_path, three_params);

        let result = app.render(full_path).await.unwrap();
        assert_eq!(result, "6");
    }
}
