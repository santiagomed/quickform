mod operation;
mod state;

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

use operation::{FunctionSignature, Operation};
use state::{State, NoState, IntoFunctionParams};

type BoxedOperation = Box<dyn Fn() -> Pin<Box<dyn Future<Output = Box<dyn std::any::Any + Send>> + Send>> + Send + Sync>;

pub struct App<T> {
    state: T,
    operations: HashMap<String, BoxedOperation>,
}

impl App<NoState> {
    pub fn new() -> Self {
        Self {
            state: NoState,
            operations: HashMap::new(),
        }
    }

    pub fn with_state<S>(self, state: S) -> App<State<S>> {
        App {
            state: State::new(state),
            operations: HashMap::new(),
        }
    }
}

// Base implementation for App with one state
impl<S1: Send + Sync + 'static> App<State<S1>> {
    pub fn with_state<S2>(self, state: S2) -> App<(State<S1>, State<S2>)> {
        App {
            state: (self.state, State::new(state)),
            operations: HashMap::new(),
        }
    }
}

// Or we could use a macro to generate these implementations:
macro_rules! impl_app_with_state {
    (($($idx:tt),*); $($prev:ident),*; $next:ident) => {
        impl<$($prev: Send + Sync + 'static,)*> App<($(State<$prev>,)*)> {
            pub fn with_state<$next>(self, state: $next) -> App<($(State<$prev>,)* State<$next>)> {
                App {
                    state: ($(self.state.$idx,)* State::new(state)),
                    operations: HashMap::new(),
                }
            }
        }
    };
}

// Generate implementations for tuples of different sizes
impl_app_with_state!((0); S1; S2);
impl_app_with_state!((0, 1); S1, S2; S3);
impl_app_with_state!((0, 1, 2); S1, S2, S3; S4);
impl_app_with_state!((0, 1, 2, 3); S1, S2, S3, S4; S5);

impl<T: Send + Sync + Clone + 'static> App<T> {
    pub fn operation<FSig, F>(mut self, name: &str, operation: F) -> Self 
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
            let fut = operation.execute(params);
            Box::pin(async move {
                let result = fut.await;
                Box::new(result) as Box<dyn std::any::Any + Send>
            }) as Pin<Box<dyn Future<Output = _> + Send>>
        };

        self.operations.insert(name.to_string(), Box::new(wrapped_op));
        self
    }

    pub async fn run_operation<R: Clone + 'static>(&self, name: &str) -> Option<R> {
        if let Some(op) = self.operations.get(name) {
            let result = op().await;
            result.downcast_ref::<R>().cloned()
        } else {
            None
        }
    }
}

// Test implementation
#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[derive(Clone)]
    struct User {
        name: String,
        _age: u32,
    }

    #[derive(Clone)]
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

        let result = app.run_operation::<String>("get_default").await;
        assert_eq!(result, Some("Default".to_string()));
    }

    #[tokio::test]
    async fn test_single_param() {
        async fn get_user_name(user: State<User>) -> String {
            user.get_ref().name.clone()
        }

        let app = App::new()
            .with_state(User {
                name: "Alice".to_string(),
                _age: 30,
            })
            .operation("get_name", get_user_name);

        let result = app.run_operation::<String>("get_name").await;
        assert_eq!(result, Some("Alice".to_string()));
    }

    #[tokio::test]
    async fn test_two_params() {
        async fn get_user_with_timeout(
            user: State<User>,
            config: State<Config>
        ) -> (String, Duration) {
            (
                user.get_ref().name.clone(),
                config.get_ref().timeout,
            )
        }

        let app = App::new()
            .with_state(User {
                name: "Bob".to_string(),
                _age: 25,
            })
            .with_state(Config {
                timeout: Duration::from_secs(30),
            })
            .operation("get_user_timeout", get_user_with_timeout);

        let result = app.run_operation::<(String, Duration)>("get_user_timeout").await;
        assert!(result.is_some());
    }

    #[tokio::test]
    async fn test_three_params() {
        async fn three_params(x: State<i32>, y: State<i32>, z: State<i32>) -> i32 {
            x.get_ref() + y.get_ref() + z.get_ref()
        }

        let app = App::new()
            .with_state(1)
            .with_state(2)
            .with_state(3)
            .operation("three_params", three_params);

        let result = app.run_operation::<i32>("three_params").await;
        assert_eq!(result, Some(6));
    }
}
