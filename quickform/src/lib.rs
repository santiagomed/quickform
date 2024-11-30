// src/lib.rs
mod operation;
mod state;
mod context;

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

use operation::{FunctionSignature, Operation};
use state::{Data, NoData, IntoFunctionParams};
use context::Context;
type BoxedOperation = Box<dyn Fn() -> Pin<Box<dyn Future<Output = Box<dyn Context>> + Send>> + Send + Sync>;

pub struct App<T> {
    state: T,
    operations: HashMap<String, BoxedOperation>,
    paths: Vec<String>,
}

impl App<NoData> {
    pub fn new() -> Self {
        Self {
            state: NoData,
            operations: HashMap::new(),
            paths: Vec::new(),
        }
    }

    pub fn with_state<S>(self, state: S) -> App<Data<S>> {
        App {
            state: Data::new(state),
            operations: self.operations,
            paths: self.paths,
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

    pub async fn run_operation(&self, name: &str) -> Option<minijinja::Value> {
        if let Some(op) = self.operations.get(name) {
            let result = op().await;
            Some(result.to_value())
        } else {
            None
        }
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        for path in &self.paths {
            if let Some(context) = self.run_operation(path).await {
                let template_src = std::fs::read_to_string(path)?;
                println!("{}", context);
                let rendered = minijinja::render!(&template_src, context);
                println!("{}", rendered);
            }
        }
        Ok(())
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

        let app = App::new()
            .operation("get_default", get_default_name);

        let result = app.run_operation("get_default").await;
        assert_eq!(result, Some(minijinja::Value::from("Default")));
    }

    #[tokio::test]
    async fn test_single_param() {
        async fn double_age(user: Data<User>) -> User {
            User {
                name: user.get_ref().name.clone(),
                age: user.get_ref().age * 2,
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
            (
                user.get_ref().name.clone(),
                config.get_ref().timeout,
            )
        }

        let app = App::new()
            .with_state(User {
                name: "Bob".to_string(),
                age: 25,
            })
            .with_state(Config {
                timeout: Duration::from_secs(30),
            })
            .operation("get_user_timeout", get_user_with_timeout);

        let result = app.run_operation("get_user_timeout").await;
        assert!(result.is_some());
    }

    #[tokio::test]
    async fn test_three_params() {
        async fn three_params(x: Data<i32>, y: Data<i32>, z: Data<i32>) -> i32 {
            x.get_ref() + y.get_ref() + z.get_ref()
        }

        let app = App::new()
            .with_state(1)
            .with_state(2)
            .with_state(3)
            .operation("three_params", three_params);

        let result = app.run_operation("three_params").await;
        assert_eq!(result, Some(minijinja::Value::from(6)));
    }
}
