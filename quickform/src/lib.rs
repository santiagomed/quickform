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
mod context;
mod error;
mod fs;
mod loader;
mod operation;
mod state;
mod template;

use std::future::Future;
use std::path::Path;
use std::pin::Pin;
use serde::Serialize;
use tokio::sync::RwLock;
use std::sync::Arc;

use context::Context;
use error::Error;
use fs::MemFS;
use operation::{FunctionSignature, Operation, OperationKind};
use state::{Data, IntoFunctionParams, NoData};
use template::TemplateEngine;

/// A type alias for Results returned by this library
type Result<T> = std::result::Result<T, Error>;

/// The main application struct that manages state, operations, and template rendering
///
/// # Type Parameters
///
/// * `T` - The type of state stored in the App
pub struct App<T> {
    state: T,
    operations: Vec<OperationKind>,
    fs: Arc<RwLock<MemFS>>,
    engine: TemplateEngine<'static>,
}

impl Default for App<NoData> {
    fn default() -> Self {
        Self {
            state: NoData,
            operations: Vec::new(),
            fs: Arc::new(RwLock::new(MemFS::new())),
            engine: TemplateEngine::new(),
        }
    }
}

impl App<NoData> {
    /// Configures the app with templates from a directory
    ///
    /// # Arguments
    ///
    /// * `template_dir` - Path to the directory containing templates
    ///
    /// # Returns
    ///
    /// * `Result<Self>` - The configured App or an error if template loading fails
    pub fn from_dir<P: AsRef<Path>>(template_dir: P) -> Self {
        let fs = MemFS::read_from_disk(template_dir).unwrap_or_default();
        let engine = TemplateEngine::from_memfs(fs.clone());
        Self {
            engine,
            fs: Arc::new(RwLock::new(fs)),
            ..Self::default()
        }
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
    /// Registers a render operation with the application
    ///
    /// # Type Parameters
    ///
    /// * `FSig` - The function signature of the operation
    /// * `F` - The operation type
    ///
    /// # Arguments
    ///
    /// * `template_path` - The path to the template file
    /// * `operation` - The operation function to register
    ///
    /// # Returns
    ///
    /// The App instance with the new operation registered
    pub fn render_operation<FSig, F>(mut self, template_path: &str, operation: F) -> Self
    where
        FSig: FunctionSignature + 'static,
        F: Operation<FSig> + Copy + Send + Sync + 'static,
        F::Future: Send + 'static,
        FSig::Output: Serialize,
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

        self.operations.push(OperationKind::Render(
            template_path.to_string(),
            Box::new(wrapped_op),
        ));
        self
    }

    /// Registers a state operation with the application
    ///
    /// # Type Parameters
    ///
    /// * `FSig` - The function signature of the operation
    /// * `F` - The operation type
    ///
    /// # Arguments
    ///
    /// * `operation` - The operation function to register
    ///
    /// # Returns
    ///
    /// The App instance with the new operation registered
    pub fn state_operation<FSig, F>(mut self, operation: F) -> Self
    where
        FSig: FunctionSignature + 'static,
        F: Operation<FSig> + Copy + Send + Sync + 'static,
        F::Future: Send + 'static,
        FSig::Output: Send + 'static,
        T: IntoFunctionParams<FSig>,
    {
        let state = self.state.clone();
        let wrapped_op = move || {
            let params = state.clone().into_params();
            let fut = operation.invoke(params);
            Box::pin(async move {
                fut.await;
                ()
            }) as Pin<Box<dyn Future<Output = ()> + Send>>
        };

        self.operations.push(OperationKind::State(Box::new(wrapped_op)));
        self
    }

    /// Executes all registered operations and renders their results
    ///
    /// # Returns
    ///
    /// * `Result<()>` - Success or an error if any operation fails
    pub async fn run<P: AsRef<Path>>(&self, output_dir: P) -> Result<()> {
        for operation in &self.operations {
            match operation {
                OperationKind::Render(template_path, op) => {
                    let context = op().await;
                    let rendered = self.engine.render(template_path, &context.to_value())?;
                    self.fs.write().await.write_file(template_path, rendered.as_bytes().to_vec())?;
                }
                OperationKind::State(op) => {
                    op().await;
                }
            }
        }
        
        self.fs.write().await.write_to_disk(output_dir.as_ref())?;
        Ok(())
    }
}

// Test implementation
#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use std::collections::HashMap;

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
        async fn get_default_name() -> HashMap<String, String> {
            let mut map = HashMap::new();
            map.insert("value".to_string(), "Default".to_string());
            map
        }

        let tmp_dir = tempdir::TempDir::new("test").unwrap();
        let template_path = tmp_dir.path().join("get_default.jinja");
        std::fs::write(&template_path, "{{ value }}").unwrap();

        let app = App::from_dir(&tmp_dir.path())
            .render_operation("get_default.jinja", get_default_name);

        let output_dir = tmp_dir.path().join("output");
        app.run(&output_dir).await.unwrap();
        assert!(output_dir.join("get_default.jinja").exists());
        assert_eq!(std::fs::read_to_string(output_dir.join("get_default.jinja")).unwrap(), "Default");
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
            let new_name = user
                .name
                .into_bytes()
                .iter()
                .map(|b| format!("{:02x}", b))
                .collect::<Vec<String>>()
                .join("-");
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

        let app = App::from_dir(&tmp_dir.path())
            .with_state(User {
                name: "Alice".to_string(),
                age: 30,
            })
            .render_operation("double_age.jinja", double_age)
            .render_operation("child/codify_name.jinja", codify_name);

        let output_dir = tmp_dir.path().join("output");
        app.run(&output_dir).await.unwrap();
        assert!(output_dir.join("double_age.jinja").exists());
        assert_eq!(std::fs::read_to_string(output_dir.join("double_age.jinja")).unwrap(), "Age: 60");
        assert!(output_dir.join("child/codify_name.jinja").exists());
        assert_eq!(std::fs::read_to_string(output_dir.join("child/codify_name.jinja")).unwrap(), "Name: 41-6c-69-63-65");
    }

    #[tokio::test]
    async fn test_multiple_params() {
        async fn get_user_with_timeout(
            user: Data<User>,
            config: Data<Config>,
        ) -> HashMap<String, String> {
            let mut map = HashMap::new();
            map.insert("user".to_string(), user.clone_inner().await.name);
            map.insert("timeout".to_string(), config.clone_inner().await.timeout.as_secs().to_string());
            map
        }

        let tmp_dir = tempdir::TempDir::new("test").unwrap();
        let template_path = tmp_dir.path().join("multiple_params.jinja");
        std::fs::write(&template_path, "{{ timeout }} {{ user }}").unwrap();

        let app = App::from_dir(&tmp_dir.path())
            .with_state(User {
                name: "Bob".to_string(),
                age: 25,
            })
            .with_state(Config {
                timeout: Duration::from_secs(30),
            })
            .render_operation("multiple_params.jinja", get_user_with_timeout);

        let output_dir = tmp_dir.path().join("output");
        app.run(&output_dir).await.unwrap();
        assert!(output_dir.join("multiple_params.jinja").exists());
        assert_eq!(std::fs::read_to_string(output_dir.join("multiple_params.jinja")).unwrap(), "30 Bob");
    }

    #[tokio::test]
    async fn test_simple_params() {
        async fn three_params(x: Data<i32>, y: Data<i32>, z: Data<i32>) -> HashMap<String, i32> {
            let x = x.clone_inner().await;
            let y = y.clone_inner().await;
            let z = z.clone_inner().await;
            let mut map = HashMap::new();
            map.insert("sum".to_string(), x + y + z);
            map
        }

        let tmp_dir = tempdir::TempDir::new("test").unwrap();
        let template_path = tmp_dir.path().join("simple_params.jinja");
        std::fs::write(&template_path, "{{ sum }}").unwrap();

        let app = App::from_dir(&tmp_dir.path())
            .with_state(1)
            .with_state(2)
            .with_state(3)
            .render_operation("simple_params.jinja", three_params);

        let output_dir = tmp_dir.path().join("output");
        app.run(&output_dir).await.unwrap();
        assert!(output_dir.join("simple_params.jinja").exists());
        assert_eq!(std::fs::read_to_string(output_dir.join("simple_params.jinja")).unwrap(), "6");
    }

    #[tokio::test]
    async fn test_state_operation_single_state() {
        let app = App::default()
            .with_state(User {
                name: "Alice".to_string(),
                age: 30,
            })
            .state_operation(|user: Data<User>| async move {
                user.update(|u| u.name = "Bob".to_string()).await;
            });

        // Run the app
        let tmp_dir = tempdir::TempDir::new("test").unwrap();
        app.run(tmp_dir.path()).await.unwrap();

        // Verify the state was updated
        assert_eq!(
            app.state.clone_inner().await.name,
            "Bob"
        );
    }

    #[tokio::test]
    async fn test_state_operation_multiple_states() {
        let app = App::default()
            .with_state(User {
                name: "Alice".to_string(),
                age: 30,
            })
            .with_state(Config {
                timeout: Duration::from_secs(30),
            })
            .state_operation(|user: Data<User>, config: Data<Config>| async move {
                user.update(|u| u.name = "Bob".to_string()).await;
                config.update(|c| c.timeout = Duration::from_secs(60)).await;
            });

        // Run the app
        let tmp_dir = tempdir::TempDir::new("test").unwrap();
        app.run(tmp_dir.path()).await.unwrap();

        // Verify both states were updated
        assert_eq!(
            app.state.0.clone_inner().await.name,
            "Bob"
        );
        assert_eq!(
            app.state.1.clone_inner().await.timeout,
            Duration::from_secs(60)
        );
    }

    #[tokio::test]
    async fn test_state_operation_chain() {
        let app = App::default()
            .with_state(User {
                name: "Alice".to_string(),
                age: 30,
            })
            .state_operation(|user: Data<User>| async move {
                user.update(|u| u.name = "Bob".to_string()).await;
            })
            .state_operation(|user: Data<User>| async move {
                let current = user.clone_inner().await;
                user.update(|u| u.name = format!("{}-modified", current.name)).await;
            });

        // Run the app
        let tmp_dir = tempdir::TempDir::new("test").unwrap();
        app.run(tmp_dir.path()).await.unwrap();

        // Verify the state was updated by both operations
        assert_eq!(
            app.state.clone_inner().await.name,
            "Bob-modified"
        );
    }

    #[tokio::test]
    async fn test_mixed_operations() {
        let tmp_dir = tempdir::TempDir::new("test").unwrap();
        let template_path = tmp_dir.path().join("user.jinja");
        std::fs::write(&template_path, "Name: {{ name }}").unwrap();

        let app = App::from_dir(&tmp_dir.path())
            .with_state(User {
                name: "Alice".to_string(),
                age: 30,
            })
            .state_operation(|user: Data<User>| async move {
                user.update(|u| u.name = "Bob".to_string()).await;
            })
            .render_operation("user.jinja", |user: Data<User>| async move {
                user.clone_inner().await
            });

        let output_dir = tmp_dir.path().join("output");
        app.run(&output_dir).await.unwrap();

        // Verify the state was updated
        assert_eq!(
            app.state.clone_inner().await.name,
            "Bob"
        );

        // Verify the template was rendered with the updated state
        assert_eq!(
            std::fs::read_to_string(output_dir.join("user.jinja")).unwrap(),
            "Name: Bob"
        );
    }
}
