// src/state.rs
use std::sync::Arc;
use std::ops::Deref;
use crate::operation::FunctionSignature;

// Data wrapper for template functions
pub struct Data<T: ?Sized>(Arc<T>);

impl<T> Data<T> {
    /// Create new `Data` instance
    pub fn new(state: T) -> Data<T> {
        Data(Arc::new(state))
    }
}

impl<T: ?Sized> Data<T> {
    /// Returns reference to inner `T`
    pub fn get_ref(&self) -> &T {
        self.0.as_ref()
    }

    /// Unwraps to the internal Arc<T>
    pub fn into_inner(self) -> Arc<T> {
        self.0
    }
}

impl<T: ?Sized> Deref for Data<T> {
    type Target = Arc<T>;

    fn deref(&self) -> &Arc<T> {
        &self.0
    }
}

impl<T: ?Sized> Clone for Data<T> {
    fn clone(&self) -> Data<T> {
        Data(Arc::clone(&self.0))
    }
}


impl<T: Clone + ?Sized> Data<T> {
    /// Returns a cloned value of the inner `T`
    pub fn get_value(&self) -> T {
        (*self.0).clone()
    }
}

impl<T: ?Sized> From<Arc<T>> for Data<T> {
    fn from(arc: Arc<T>) -> Self {
        Data(arc)
    }
}

#[derive(Default, Clone)]
pub struct NoData;


// Trait for converting stored states into function parameters
pub trait IntoFunctionParams<F: FunctionSignature> {
    fn into_params(self) -> F::Params;
}

macro_rules! impl_into_function_params {
    // Base case with no parameters
    () => {
        impl<F> IntoFunctionParams<F> for NoData 
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
        impl<$T, F> IntoFunctionParams<F> for Data<$T>
        where
            F: FunctionSignature<Params = Data<$T>>,
            $T: Clone + 'static,
        {
            fn into_params(self) -> F::Params {
                self
            }
        }
    };

    // Case for multiple parameters
    ($($T:ident),+) => {
        impl<$($T,)+ F> IntoFunctionParams<F> for ($(Data<$T>,)+)
        where
            F: FunctionSignature<Params = ($(Data<$T>,)+)>,
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
        // Test NoData
        let no_state = NoData;
        let _: () = <NoData as IntoFunctionParams<fn() -> std::future::Ready<()>>>::into_params(no_state);

        // Test single state
        let state = Data::new(User { _name: "Alice".to_string() });
        let _: Data<User> = <Data<User> as IntoFunctionParams<fn(Data<User>) -> std::future::Ready<Data<User>>>>::into_params(state);

        // Test two states
        let user_state = Data::new(User { _name: "Bob".to_string() });
        let config_state = Data::new(Config { _timeout: Duration::from_secs(30) });
        let states = (user_state, config_state);
        let _: (Data<User>, Data<Config>) = <(Data<User>, Data<Config>) as IntoFunctionParams<fn((Data<User>, Data<Config>)) -> std::future::Ready<(Data<User>, Data<Config>)>>>::into_params(states);
    }
}