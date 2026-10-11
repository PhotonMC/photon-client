//! JSON text embedded in a case (a chat component, a server status) keeps
//! its keys in the order they were written and uses `", "` and `": "`
//! between items, as the scenarios spell it.

use crate::line::{float_text, write_string};

/// A JSON value whose object keys keep their order.
#[derive(Clone, Debug)]
pub enum Ordered {
    Text(String),
    Bool(bool),
    Int(i64),
    Float(f64),
    List(Vec<Ordered>),
    Object(Vec<(String, Ordered)>),
}

impl From<&str> for Ordered {
    fn from(value: &str) -> Self {
        Self::Text(value.to_owned())
    }
}

impl From<String> for Ordered {
    fn from(value: String) -> Self {
        Self::Text(value)
    }
}

impl From<bool> for Ordered {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<i64> for Ordered {
    fn from(value: i64) -> Self {
        Self::Int(value)
    }
}

impl From<f64> for Ordered {
    fn from(value: f64) -> Self {
        Self::Float(value)
    }
}

impl From<Vec<Ordered>> for Ordered {
    fn from(value: Vec<Ordered>) -> Self {
        Self::List(value)
    }
}

/// `object(&[("key", value), ...])` with the keys in this order.
pub fn object(pairs: Vec<(&str, Ordered)>) -> Ordered {
    Ordered::Object(pairs.into_iter().map(|(key, value)| (key.to_owned(), value)).collect())
}

impl Ordered {
    /// The value as JSON text.
    pub fn dumps(&self) -> String {
        let mut out = String::new();
        self.write(&mut out);
        out
    }

    /// Adds `key` at the end of an object.
    pub fn with(mut self, key: &str, value: impl Into<Ordered>) -> Self {
        if let Self::Object(pairs) = &mut self {
            pairs.push((key.to_owned(), value.into()));
        }
        self
    }

    fn write(&self, out: &mut String) {
        match self {
            Self::Text(text) => write_string(out, text),
            Self::Bool(flag) => out.push_str(if *flag { "true" } else { "false" }),
            Self::Int(value) => out.push_str(&value.to_string()),
            Self::Float(value) => out.push_str(&float_text(*value)),
            Self::List(items) => write_list(out, items),
            Self::Object(pairs) => write_object(out, pairs),
        }
    }
}

fn write_list(out: &mut String, items: &[Ordered]) {
    out.push('[');
    for (index, item) in items.iter().enumerate() {
        if index > 0 {
            out.push_str(", ");
        }
        item.write(out);
    }
    out.push(']');
}

fn write_object(out: &mut String, pairs: &[(String, Ordered)]) {
    out.push('{');
    for (index, (key, value)) in pairs.iter().enumerate() {
        if index > 0 {
            out.push_str(", ");
        }
        write_string(out, key);
        out.push_str(": ");
        value.write(out);
    }
    out.push('}');
}
