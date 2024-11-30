use minijinja::Value;
use serde::Serialize;

pub trait Context {
    fn to_value(&self) -> Value;
}

impl<T: Serialize> Context for T {
    fn to_value(&self) -> Value {
        Value::from_serialize(self)
    }
}