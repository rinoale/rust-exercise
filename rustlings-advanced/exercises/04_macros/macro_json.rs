// Typing lecture: declarative macros.
//
// A macro can match Rust token trees and recursively expand them. This exercise
// builds a tiny JSON-like value enum with a `json!` macro.
//
// TODO: Replace the catch-all macro with real pattern arms.

use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
enum Json {
    Null,
    Bool(bool),
    Num(f64),
    Str(String),
    Array(Vec<Json>),
    Object(HashMap<String, Json>),
}

impl From<bool> for Json {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<i32> for Json {
    fn from(value: i32) -> Self {
        Self::Num(value as f64)
    }
}

impl From<f64> for Json {
    fn from(value: f64) -> Self {
        Self::Num(value)
    }
}

impl From<&str> for Json {
    fn from(value: &str) -> Self {
        Self::Str(value.to_string())
    }
}

macro_rules! json {
    ($($tokens:tt)*) => {
        todo!("match JSON-like syntax and build Json values")
    };
}

fn main() {
    println!("{:?}", json!({"course": "advanced"}));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_scalar_values() {
        assert_eq!(json!(null), Json::Null);
        assert_eq!(json!(true), Json::Bool(true));
        assert_eq!(json!(42), Json::Num(42.0));
        assert_eq!(json!("hello"), Json::Str(String::from("hello")));
    }

    #[test]
    fn builds_arrays() {
        assert_eq!(
            json!([1, true, "three"]),
            Json::Array(vec![
                Json::Num(1.0),
                Json::Bool(true),
                Json::Str(String::from("three"))
            ])
        );
    }

    #[test]
    fn builds_objects() {
        let value = json!({"name": "Rust", "year": 2015});

        let Json::Object(object) = value else {
            panic!("expected object");
        };

        assert_eq!(object.get("name"), Some(&Json::Str(String::from("Rust"))));
        assert_eq!(object.get("year"), Some(&Json::Num(2015.0)));
    }
}
