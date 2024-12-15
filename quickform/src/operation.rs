//! Operation traits and implementations for handling async function signatures
//!
//! This module provides the core traits and implementations for working with
//! async functions in the QuickForm framework. It handles functions with
//! different numbers of parameters (0 to 4) through macro-generated implementations.
//!
//! # Examples
//!
//! ```rust
//! use quickform::operation::*;
//!
//! // Example async function
//! async fn greet(name: String) -> String {
//!     format!("Hello, {}!", name)
//! }
//!
//! // Using the Operation trait
//! let result = greet.invoke("Alice".to_string()).await;
//! assert_eq!(result, "Hello, Alice!");
//! ```

use std::future::Future;
use std::pin::Pin;

use crate::context::Context;

// Operation that returns context for template rendering
type BoxedRenderOperation =
    Box<dyn Fn() -> Pin<Box<dyn Future<Output = Box<dyn Context>> + Send>> + Send + Sync>;

// Operation that only modifies state
type BoxedStateOperation =
    Box<dyn Fn() -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>;

// Enum to store both types of operations
pub enum OperationKind {
    Render(String,BoxedRenderOperation), // Include template path
    State(BoxedStateOperation),
}

/// Defines the signature of a function, including its parameter and output types
///
/// This trait is implemented for function pointers that return futures,
/// allowing the framework to work with their parameter and return types in a generic way.
pub trait FunctionSignature {
    /// The type of parameters the function accepts
    type Params;
    /// The type that the function's future resolves to
    type Output;
}

/// Defines how to invoke an operation with the given parameter types
///
/// This trait is implemented for closures and functions that match
/// the signature defined by a `FunctionSignature`.
///
/// # Type Parameters
///
/// * `F` - The function signature this operation implements
pub trait Operation<F: FunctionSignature> {
    /// The future type returned by this operation
    type Future: Future<Output = F::Output>;

    /// Invokes the operation with the given parameters
    ///
    /// # Arguments
    ///
    /// * `params` - The parameters to pass to the operation
    ///
    /// # Returns
    ///
    /// A future that will resolve to the operation's output
    fn invoke(&self, params: F::Params) -> Self::Future;
}

// Macro to generate implementations for both traits
macro_rules! impl_function_traits {
    // Base case with no parameters
    () => {
        impl<Fut, Out> FunctionSignature for fn() -> Fut
        where
            Fut: Future<Output = Out>
        {
            type Params = ();
            type Output = Out;
        }

        impl<F, Fut> Operation<fn() -> Fut> for F
        where
            F: Fn() -> Fut,
            Fut: Future,
        {
            type Future = Fut;
            fn invoke(&self, _: ()) -> Self::Future {
                self()
            }
        }
    };

    // Case for 1 parameter
    (($T:ident, $idx:tt)) => {
        impl<$T, Fut, Out> FunctionSignature for fn($T) -> Fut
        where
            Fut: Future<Output = Out>
        {
            type Params = $T;
            type Output = Out;
        }

        impl<F, $T, Fut> Operation<fn($T) -> Fut> for F
        where
            F: Fn($T) -> Fut,
            Fut: Future,
        {
            type Future = Fut;
            fn invoke(&self, $idx: $T) -> Self::Future {
                self($idx)
            }
        }
    };

    // Case for N parameters
    ($(($T:ident, $idx:tt)),+) => {
        impl<$($T,)+ Fut, Out> FunctionSignature for fn($($T),+) -> Fut
        where
            Fut: Future<Output = Out>
        {
            type Params = ($($T,)+);
            type Output = Out;
        }

        impl<F, $($T,)+ Fut> Operation<fn($($T),+) -> Fut> for F
        where
            F: Fn($($T),+) -> Fut,
            Fut: Future,
        {
            type Future = Fut;
            fn invoke(&self, ($($idx,)+): ($($T,)+)) -> Self::Future {
                self($($idx,)+)
            }
        }
    };
}

// Generate implementations for different arities
impl_function_traits!(); // 0 parameters
impl_function_traits!((T1, p1)); // 1 parameter
impl_function_traits!((T1, p1), (T2, p2)); // 2 parameters
impl_function_traits!((T1, p1), (T2, p2), (T3, p3)); // 3 parameters
impl_function_traits!((T1, p1), (T2, p2), (T3, p3), (T4, p4)); // 4 parameters

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_different_arities() {
        async fn no_params() -> i32 {
            42
        }
        async fn one_param(x: i32) -> i32 {
            x + 1
        }
        async fn two_params(x: i32, y: i32) -> i32 {
            x + y
        }
        async fn three_params(x: i32, y: i32, z: i32) -> i32 {
            x + y + z
        }

        let _f0: fn() -> _ = no_params;
        let _f1: fn(i32) -> _ = one_param;
        let _f2: fn(i32, i32) -> _ = two_params;
        let _f3: fn(i32, i32, i32) -> _ = three_params;

        assert_eq!(no_params.invoke(()).await, 42);
        assert_eq!(one_param.invoke(1).await, 2);
        assert_eq!(two_params.invoke((1, 2)).await, 3);
        assert_eq!(three_params.invoke((1, 2, 3)).await, 6);
    }
}
