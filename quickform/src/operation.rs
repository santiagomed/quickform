use std::future::Future;

pub trait FunctionSignature {
    type Params;
    type Output;
}

pub trait Operation<F: FunctionSignature> {
    type Future: Future<Output = F::Output>;
    fn execute(&self, params: F::Params) -> Self::Future;
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
            fn execute(&self, _: ()) -> Self::Future {
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
            fn execute(&self, $idx: $T) -> Self::Future {
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
            fn execute(&self, ($($idx,)+): ($($T,)+)) -> Self::Future {
                self($($idx,)+)
            }
        }
    };
}

// Generate implementations for different arities
impl_function_traits!();                                    // 0 parameters
impl_function_traits!((T1, p1));                           // 1 parameter
impl_function_traits!((T1, p1), (T2, p2));                 // 2 parameters
impl_function_traits!((T1, p1), (T2, p2), (T3, p3));      // 3 parameters
impl_function_traits!((T1, p1), (T2, p2), (T3, p3), (T4, p4)); // 4 parameters

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_different_arities() {
        async fn no_params() -> i32 { 42 }
        async fn one_param(x: i32) -> i32 { x + 1 }
        async fn two_params(x: i32, y: i32) -> i32 { x + y }
        async fn three_params(x: i32, y: i32, z: i32) -> i32 { x + y + z }

        let _f0: fn() -> _ = no_params;
        let _f1: fn(i32) -> _ = one_param;
        let _f2: fn(i32, i32) -> _ = two_params;
        let _f3: fn(i32, i32, i32) -> _ = three_params;

        assert_eq!(no_params.execute(()).await, 42);
        assert_eq!(one_param.execute(1).await, 2);
        assert_eq!(two_params.execute((1, 2)).await, 3);
        assert_eq!(three_params.execute((1, 2, 3)).await, 6);
    }
}