use std::sync::Arc;
use std::ops::Deref;
use crate::operation::FunctionSignature;

// State wrapper for template functions
pub struct State<T: ?Sized>(Arc<T>);

impl<T> State<T> {
    /// Create new `State` instance
    pub fn new(state: T) -> State<T> {
        State(Arc::new(state))
    }
}

impl<T: ?Sized> State<T> {
    /// Returns reference to inner `T`
    pub fn get_ref(&self) -> &T {
        self.0.as_ref()
    }

    /// Unwraps to the internal Arc<T>
    pub fn into_inner(self) -> Arc<T> {
        self.0
    }
}

impl<T: ?Sized> Deref for State<T> {
    type Target = Arc<T>;

    fn deref(&self) -> &Arc<T> {
        &self.0
    }
}

impl<T: ?Sized> Clone for State<T> {
    fn clone(&self) -> State<T> {
        State(Arc::clone(&self.0))
    }
}


impl<T: Clone + ?Sized> State<T> {
    /// Returns a cloned value of the inner `T`
    pub fn get_value(&self) -> T {
        (*self.0).clone()
    }
}

impl<T: ?Sized> From<Arc<T>> for State<T> {
    fn from(arc: Arc<T>) -> Self {
        State(arc)
    }
}

#[derive(Default, Clone)]
pub struct NoState;


// Trait for converting stored states into function parameters
pub trait IntoFunctionParams<F: FunctionSignature> {
    fn into_params(self) -> F::Params;
}

macro_rules! impl_into_function_params {
    // Base case with no parameters
    () => {
        impl<F> IntoFunctionParams<F> for NoState 
        where
            F: FunctionSignature<Params = ()>
        {
            fn into_params(self) -> F::Params {
                ()
            }
        }
    };

    // Case for single parameter
    ($T:ident) => {
        impl<$T, F> IntoFunctionParams<F> for State<$T>
        where
            F: FunctionSignature<Params = State<$T>>,
            $T: Clone + 'static,
        {
            fn into_params(self) -> F::Params {
                self
            }
        }
    };

    // Case for multiple parameters
    ($($T:ident),+) => {
        impl<$($T,)+ F> IntoFunctionParams<F> for ($(State<$T>,)+)
        where
            F: FunctionSignature<Params = ($(State<$T>,)+)>,
            $($T: Clone + 'static,)+
        {
            fn into_params(self) -> F::Params {
                self
            }
        }
    };
}

// Generate implementations for different arities
impl_into_function_params!();                          // 0 parameters
impl_into_function_params!(S1);                      // 1 parameter
impl_into_function_params!(S1, S2);                // 2 parameters
impl_into_function_params!(S1, S2, S3);          // 3 parameters
impl_into_function_params!(S1, S2, S3, S4);    // 4 parameters

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[derive(Clone)]
    struct User {
        _name: String,
    }

    #[derive(Clone)]
    struct Config {
        _timeout: Duration,
    }

    #[test]
    fn test_into_params() {
        // Test NoState
        let no_state = NoState;
        let _: () = <NoState as IntoFunctionParams<fn() -> std::future::Ready<()>>>::into_params(no_state);

        // Test single state
        let state = State::new(User { _name: "Alice".to_string() });
        let _: State<User> = <State<User> as IntoFunctionParams<fn(State<User>) -> std::future::Ready<State<User>>>>::into_params(state);

        // Test two states
        let user_state = State::new(User { _name: "Bob".to_string() });
        let config_state = State::new(Config { _timeout: Duration::from_secs(30) });
        let states = (user_state, config_state);
        let _: (State<User>, State<Config>) = <(State<User>, State<Config>) as IntoFunctionParams<fn((State<User>, State<Config>)) -> std::future::Ready<(State<User>, State<Config>)>>>::into_params(states);
    }
}