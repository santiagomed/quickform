//! State management for QuickForm
//!
//! This module provides types and traits for managing application state in a thread-safe
//! and type-safe manner. It includes wrapper types for state data and traits for
//! converting state into function parameters.
//!
//! # Examples
//!
//! ```rust
//! use quickform::state::{Data, NoData};
//!
//! #[derive(Clone)]
//! struct User {
//!     name: String,
//! }
//!
//! // Create state wrapper
//! let user_state = Data::new(User { name: "Alice".to_string() });
//!
//! // Clone the state (only clones the Arc, not the inner data)
//! let user_state_clone = user_state.clone();
//!
//! // Access the inner data
//! assert_eq!(user_state.get_ref().name, "Alice");
//! ```

use std::sync::Arc;
use std::ops::Deref;
use crate::operation::FunctionSignature;

/// Thread-safe wrapper for state data
///
/// Wraps any type T in an Arc for thread-safe reference counting.
/// The type parameter T can be unsized (indicated by ?Sized).
///
/// # Type Parameters
///
/// * `T` - The type of state being wrapped
pub struct Data<T: ?Sized>(Arc<T>);

impl<T> Data<T> {
    /// Creates a new `Data` instance wrapping the provided state
    ///
    /// # Arguments
    ///
    /// * `state` - The state to wrap
    ///
    /// # Examples
    ///
    /// ```rust
    /// let state = Data::new(String::from("hello"));
    /// ```
    pub fn new(state: T) -> Data<T> {
        Data(Arc::new(state))
    }
}

impl<T: ?Sized> Data<T> {
    /// Returns a reference to the wrapped state
    ///
    /// # Returns
    ///
    /// A reference to the inner value
    pub fn get_ref(&self) -> &T {
        self.0.as_ref()
    }

    /// Unwraps the Data wrapper, returning the internal Arc
    ///
    /// # Returns
    ///
    /// The underlying Arc<T>
    pub fn into_inner(self) -> Arc<T> {
        self.0
    }
}

/// Implements [Deref] to allow transparent access to the underlying [Arc]
///
/// This implementation enables using methods from [Arc] directly on `Data<T>` instances
/// through deref coercion.
///
/// # Examples
///
/// ```rust
/// let data = Data::new(String::from("hello"));
/// assert_eq!(data.strong_count(), 1); // Calls Arc::strong_count through deref
/// ```
impl<T: ?Sized> Deref for Data<T> {
    type Target = Arc<T>;

    fn deref(&self) -> &Arc<T> {
        &self.0
    }
}

/// Implements [Clone] for thread-safe cloning of the state wrapper
///
/// This implementation only clones the [Arc] pointer, not the underlying data,
/// making it very efficient.
///
/// # Examples
///
/// ```rust
/// let data = Data::new(String::from("hello"));
/// let cloned = data.clone();
/// assert_eq!(data.strong_count(), 2);
/// ```
impl<T: ?Sized> Clone for Data<T> {
    fn clone(&self) -> Data<T> {
        Data(Arc::clone(&self.0))
    }
}

/// Additional functionality for `Data<T>` when T implements [Clone]
impl<T: Clone + ?Sized> Data<T> {
    /// Returns a cloned value of the inner `T`
    ///
    /// Unlike [Clone::clone], this method clones the actual data inside the [Arc],
    /// not just the reference counting wrapper.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let data = Data::new(String::from("hello"));
    /// let value = data.get_value();
    /// assert_eq!(value, "hello");
    /// ```
    pub fn get_value(&self) -> T {
        (*self.0).clone()
    }
}

/// Implements conversion from [Arc] to `Data<T>`
///
/// This allows creating a `Data<T>` instance from an existing [Arc],
/// which is useful when integrating with other code that uses [Arc] directly.
///
/// # Examples
///
/// ```rust
/// let arc = Arc::new(String::from("hello"));
/// let data: Data<String> = Data::from(arc);
/// assert_eq!(data.get_ref(), "hello");
/// ```
impl<T: ?Sized> From<Arc<T>> for Data<T> {
    fn from(arc: Arc<T>) -> Self {
        Data(arc)
    }
}

/// Represents the absence of state data
///
/// Used when an operation doesn't require any state parameters.
#[derive(Default, Clone)]
pub struct NoData;


/// Converts stored states into function parameters
///
/// This trait enables conversion of state types into the parameter types
/// expected by operation functions.
///
/// # Type Parameters
///
/// * `F` - The function signature that defines the parameter types
pub trait IntoFunctionParams<F: FunctionSignature> {
    /// Converts self into the parameter types expected by the function
    fn into_params(self) -> F::Params;
}

// Macro for implementing IntoFunctionParams for different arities
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

// The following implementations are generated:
//
// - NoData -> ()
// - Data<T> -> Data<T>
// - (Data<T1>, Data<T2>) -> (Data<T1>, Data<T2>)
// - (Data<T1>, Data<T2>, Data<T3>) -> (Data<T1>, Data<T2>, Data<T3>)
// - (Data<T1>, Data<T2>, Data<T3>, Data<T4>) -> (Data<T1>, Data<T2>, Data<T3>, Data<T4>)
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