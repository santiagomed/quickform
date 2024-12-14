//! QuickForm: A flexible templating and operation execution framework
//! 
//! This library provides a type-safe, state-aware templating system that can execute
//! async operations and render their results using templates. It's designed to be
//! flexible and extensible while maintaining strong type safety.
//!
//! # Examples
//!
//! ```rust
//! use quickform::{App};
//!
//! // Define some state
//! #[derive(Clone)]
//! struct User {
//!     name: String,
//!     age: u32,
//! }
//!
//! // Define an async operation
//! async fn process_user(user: Data<User>) -> String {
//!     format!("Hello, {}!", user.name)
//! }
//!
//! // Create and run the app
//! let app = App::new()
//!     .with_templates("templates/")
//!     .with_state(User {
//!         name: "Alice".to_string(),
//!         age: 30,
//!     })
//!     .operation("greet.txt", process_user);
//! ```
//!
//! # Features
//!
//! - **State Management**: Support for multiple state types using a type-safe API
//! - **Async Operations**: Execute async functions with state-aware parameters
//! - **Template Rendering**: Integrate with template files for output generation
//! - **Builder Pattern**: Fluent API for configuration and setup
//!
//! # Type Parameters
//!
//! - `T`: The type of state stored in the App. Can be:
//!   - `NoData`: For apps with no state
//!   - `Data<S>`: For apps with a single state type
//!   - `(Data<S1>, Data<S2>, ...)`: For apps with multiple state types
mod operation;
mod state;
mod context;
mod error;
mod fs;
mod template;

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::path::Path;

use fs::MemFS;
use template::TemplateEngine;
use operation::{FunctionSignature, Operation};
use state::{Data, NoData, IntoFunctionParams};
use context::Context;
use error::Error;

/// A type alias for Results returned by this library
type Result<T> = std::result::Result<T, Error>;

/// A boxed operation that can be executed asynchronously
type BoxedOperation = Box<dyn Fn() -> Pin<Box<dyn Future<Output = Box<dyn Context>> + Send>> + Send + Sync>;

/// The main application struct that manages state, operations, and template rendering
///
/// # Type Parameters
///
/// * `T` - The type of state stored in the App
pub struct App<T> {
    state: T,
    operations: HashMap<String, BoxedOperation>,
    paths: Vec<String>,
    fs: MemFS,
    engine: TemplateEngine<'static>,
}

impl App<NoData> {
    /// Creates a new App instance with no state
    ///
    /// # Examples
    ///
    /// ```rust
    /// let app = App::new();
    /// ```
    pub fn new() -> Self {
        Self {
            state: NoData,  
            operations: HashMap::new(),
            paths: Vec::new(),
            fs: MemFS::new(),
            engine: TemplateEngine::new(),
        }
    }

    /// Configures the app with templates from a directory
    ///
    /// # Arguments
    ///
    /// * `template_dir` - Path to the directory containing templates
    ///
    /// # Returns
    ///
    /// * `Result<Self>` - The configured App or an error if template loading fails
    pub fn with_templates<P: AsRef<Path>>(self, template_dir: P) -> Result<Self> {
        let engine = TemplateEngine::with_dir(template_dir);
        Ok(Self {
            state: NoData,
            operations: self.operations,
            paths: self.paths,
            fs: self.fs,
            engine,
        })
    }

    /// Adds state to the application
    ///
    /// # Type Parameters
    ///
    /// * `S` - The type of state to add
    ///
    /// # Arguments
    ///
    /// * `state` - The state instance to add
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

impl_app_with_state!((0); S1; S2);
impl_app_with_state!((0, 1); S1, S2; S3);
impl_app_with_state!((0, 1, 2); S1, S2, S3; S4);

impl<T: Send + Sync + Clone + 'static> App<T> {
    /// Registers an operation with the application
    ///
    /// # Type Parameters
    ///
    /// * `FSig` - The function signature of the operation
    /// * `F` - The operation type
    ///
    /// # Arguments
    ///
    /// * `name` - The name/path of the operation
    /// * `operation` - The operation function to register
    ///
    /// # Returns
    ///
    /// The App instance with the new operation registered
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
            let fut = operation.invoke(params);
            Box::pin(async move {
                let result = fut.await;
                Box::new(result) as Box<dyn Context>
            }) as Pin<Box<dyn Future<Output = _> + Send>>
        };

        self.operations.insert(name.to_string(), Box::new(wrapped_op));
        self.paths.push(name.to_string());
        self
    }

    /// Executes all registered operations and renders their results
    ///
    /// # Returns
    ///
    /// * `Result<()>` - Success or an error if any operation fails
    pub async fn run(&self) -> Result<()> {
        for path in &self.paths {
            self.render(path).await?;
        }
        Ok(())
    }
    
    /// Renders a single operation by name
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the operation to render
    ///
    /// # Returns
    ///
    /// * `Result<String>` - The rendered result or an error
    async fn render(&self, name: &str) -> Result<String> {
        println!("Rendering {}", name);
        if let Some(op) = self.operations.get(name) {
            let context = op().await.to_value();
            let rendered = self.engine.render(name, &context)?;
            Ok(rendered)
        } else {
            Err(Error::IOError(std::io::Error::new(std::io::ErrorKind::NotFound, format!("Operation not found: {}", name))))
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
    async fn test_from_dir() {
        async fn double_age(user: Data<User>) -> User {
            let user = user.clone_inner().await;
            User {
                name: user.name,
                age: user.age * 2,
            }
        }

        async fn codify_name(user: Data<User>) -> User {
            let user = user.clone_inner().await;
            let new_name = user.name.into_bytes().iter().map(|b| format!("{:02x}", b)).collect::<Vec<String>>().join("-");
            User {
                name: new_name,
                age: user.age,
            }
        }

        let tmp_dir = tempdir::TempDir::new("test").unwrap();
        
        // Create child directory
        let child_dir = tmp_dir.path().join("child");
        std::fs::create_dir(&child_dir).unwrap();
        
        let template_path_double_age = tmp_dir.path().join("double_age.jinja");
        let template_path_codify_name = child_dir.join("codify_name.jinja");
        
        std::fs::write(&template_path_double_age, "Age: {{ age }}").unwrap();
        std::fs::write(&template_path_codify_name, "Name: {{ name }}").unwrap();
        
        let app = App::new()
            .with_templates(tmp_dir.path())
            .unwrap()
            .with_state(User {
                name: "Alice".to_string(),
                age: 30,
            })
            .operation("double_age.jinja", double_age)
            .operation("child/codify_name.jinja", codify_name);

        app.run().await.unwrap();
    }

    #[tokio::test]
    async fn test_multiple_params() {
        async fn get_user_with_timeout(
            user: Data<User>,
            config: Data<Config>
        ) -> (String, Duration) {
            let user = user.clone_inner().await;
            let config = config.clone_inner().await;
            (user.name.clone(), config.timeout)
        }

        let tmp_dir = tempdir::TempDir::new("test").unwrap();
        let template_path = tmp_dir.path().join("multiple_params.jinja");
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
    async fn test_simple_params() {
        async fn three_params(x: Data<i32>, y: Data<i32>, z: Data<i32>) -> i32 {
            let x = x.clone_inner().await;
            let y = y.clone_inner().await;
            let z = z.clone_inner().await;
            x + y + z
        }

        let tmp_dir = tempdir::TempDir::new("test").unwrap();
        let template_path = tmp_dir.path().join("simple_params.jinja");
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
