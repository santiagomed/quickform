use minijinja::Value;
use serde::Serialize;

/// A trait for converting types into minijinja template values
///
/// This trait provides a uniform way to convert Rust types into values that can be
/// used in minijinja templates. It's automatically implemented for all types that
/// implement [Serialize].
///
/// # Examples
///
/// ```rust
/// use serde::Serialize;
/// 
/// #[derive(Serialize)]
/// struct User {
///     name: String,
///     age: u32,
/// }
///
/// let user = User {
///     name: String::from("Alice"),
///     age: 30,
/// };
///
/// // Convert to minijinja Value for template rendering
/// let template_value = user.to_value();
/// ```
pub trait Context {
    /// Converts the implementing type into a minijinja [Value]
    fn to_value(&self) -> Value;
}

/// Blanket implementation for all types that implement [Serialize]
///
/// This allows any type that can be serialized to be automatically used
/// as a template context.
impl<T: Serialize> Context for T {
    fn to_value(&self) -> Value {
        Value::from_serialize(self)
    }
}