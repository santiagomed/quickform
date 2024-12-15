//! State management for QuickForm
//!
//! This module provides types and traits for managing mutable application state in a thread-safe
//! and type-safe manner. It includes wrapper types for state data and traits for
//! converting state into function parameters. The state can be safely modified across
//! different tasks using Tokio's async mutex.
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
//! // Access and modify the inner data
//! async {
//!     assert_eq!(user_state.clone_inner().await.name, "Alice");
//!     user_state.update(|user| user.name = "Bob".to_string()).await;
//!     assert_eq!(user_state.clone_inner().await.name, "Bob");
//! };
//! ```

use crate::operation::FunctionSignature;
use std::ops::Deref;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Thread-safe wrapper for mutable state data
///
/// Wraps any type T in an Arc<Mutex> for thread-safe mutable access.
/// Provides an ergonomic API for accessing and modifying the state without
/// directly handling locks.
///
/// # Type Parameters
///
/// * `T` - The type of state being wrapped
pub struct Data<T>(Arc<Mutex<T>>);

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
        Data(Arc::new(Mutex::new(state)))
    }

    /// Gets a clone of the current state value
    ///
    /// # Returns
    ///
    /// A clone of the inner value
    ///
    /// # Examples
    ///
    /// ```rust
    /// let state = Data::new(String::from("hello"));
    /// async {
    ///     let value = state.clone_inner().await;
    ///     assert_eq!(value, "hello");
    /// };
    /// ```
    pub async fn clone_inner(&self) -> T
    where
        T: Clone,
    {
        self.0.lock().await.clone()
    }

    /// Updates the state using a closure
    ///
    /// # Arguments
    ///
    /// * `f` - A closure that receives a mutable reference to the state
    ///
    /// # Examples
    ///
    /// ```rust
    /// let state = Data::new(String::from("hello"));
    /// async {
    ///     state.update(|s| s.push_str(" world")).await;
    ///     assert_eq!(state.clone_inner().await, "hello world");
    /// };
    /// ```
    pub async fn update<F>(&self, f: F)
    where
        F: FnOnce(&mut T),
    {
        let mut lock = self.0.lock().await;
        f(&mut *lock);
    }

    /// Sets the state to a new value
    ///
    /// # Arguments
    ///
    /// * `new_state` - The new state value
    ///
    /// # Examples
    ///
    /// ```rust
    /// let state = Data::new(String::from("hello"));
    /// async {
    ///     state.set(String::from("world")).await;
    ///     assert_eq!(state.clone_inner().await, "world");
    /// };
    /// ```
    pub async fn set(&self, new_state: T) {
        *self.0.lock().await = new_state;
    }

    /// Unwraps the Data wrapper, returning the internal Arc<Mutex>
    ///
    /// # Returns
    ///
    /// The underlying Arc<Mutex<T>>
    pub fn into_inner(self) -> Arc<Mutex<T>> {
        self.0
    }
}

/// Implements [Deref] to allow transparent access to the underlying [Arc]
///
/// This implementation enables using methods from [Arc] directly on `Data<T>` instances
/// through deref coercion.
impl<T> Deref for Data<T> {
    type Target = Arc<Mutex<T>>;

    fn deref(&self) -> &Arc<Mutex<T>> {
        &self.0
    }
}

/// Implements [Clone] for thread-safe cloning of the state wrapper
///
/// This implementation only clones the [Arc] pointer, not the underlying data,
/// making it very efficient.
impl<T> Clone for Data<T> {
    fn clone(&self) -> Data<T> {
        Data(Arc::clone(&self.0))
    }
}

/// Implements conversion from Arc<Mutex> to `Data<T>`
///
/// This allows creating a `Data<T>` instance from an existing Arc<Mutex>,
/// which is useful when integrating with other code that uses Arc<Mutex> directly.
impl<T> From<Arc<Mutex<T>>> for Data<T> {
    fn from(arc: Arc<Mutex<T>>) -> Self {
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
            $T: Clone + Send + 'static,
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
            $($T: Clone + Send + 'static,)+
        {
            fn into_params(self) -> F::Params {
                self
            }
        }
    };
}

// Implementation for different parameter counts
impl_into_function_params!();
impl_into_function_params!(S1);
impl_into_function_params!(S1, S2);
impl_into_function_params!(S1, S2, S3);
impl_into_function_params!(S1, S2, S3, S4);

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[derive(Clone)]
    struct User {
        name: String,
    }

    #[derive(Clone)]
    struct Config {
        timeout: Duration,
    }

    #[tokio::test]
    async fn test_state_operations() {
        // Test basic state operations
        let state = Data::new(User {
            name: "Alice".to_string(),
        });
        assert_eq!(state.clone_inner().await.name, "Alice");

        state.update(|user| user.name = "Bob".to_string()).await;
        assert_eq!(state.clone_inner().await.name, "Bob");

        state
            .set(User {
                name: "Charlie".to_string(),
            })
            .await;
        assert_eq!(state.clone_inner().await.name, "Charlie");
    }

    #[tokio::test]
    async fn test_multiple_states() {
        let user_state = Data::new(User {
            name: "Alice".to_string(),
        });
        let config_state = Data::new(Config {
            timeout: Duration::from_secs(30),
        });

        // Test concurrent access
        let user_clone = user_state.clone();
        let config_clone = config_state.clone();

        let handle = tokio::spawn(async move {
            user_clone
                .update(|user| user.name = "Bob".to_string())
                .await;
            config_clone
                .update(|config| config.timeout = Duration::from_secs(60))
                .await;
        });

        // Meanwhile, read from the original references
        let user = user_state.clone_inner().await;
        let config = config_state.clone_inner().await;

        // Verify initial state
        assert_eq!(user.name, "Alice");
        assert_eq!(config.timeout, Duration::from_secs(30));

        handle.await.unwrap();

        // Verify updates
        assert_eq!(user_state.clone_inner().await.name, "Bob");
        assert_eq!(
            config_state.clone_inner().await.timeout,
            Duration::from_secs(60)
        );
    }

    #[test]
    fn test_into_params() {
        // Test NoData
        let no_state = NoData;
        let _: () =
            <NoData as IntoFunctionParams<fn() -> std::future::Ready<()>>>::into_params(no_state);

        // Test single state
        let state = Data::new(User {
            name: "Alice".to_string(),
        });
        let _: Data<User> = <Data<User> as IntoFunctionParams<
            fn(Data<User>) -> std::future::Ready<Data<User>>,
        >>::into_params(state);

        // Test two states
        let user_state = Data::new(User {
            name: "Bob".to_string(),
        });
        let config_state = Data::new(Config {
            timeout: Duration::from_secs(30),
        });
        let states = (user_state, config_state);
        let _: (Data<User>, Data<Config>) = <(Data<User>, Data<Config>) as IntoFunctionParams<
            fn((Data<User>, Data<Config>)) -> std::future::Ready<(Data<User>, Data<Config>)>,
        >>::into_params(states);
    }
}
