//! Resource identifiers used by protocol payloads.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParseError;

/// A Java-style resource identifier with a namespace and path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identifier {
    namespace: String,
    path: String,
}

impl Identifier {
    pub const DEFAULT_NAMESPACE: &'static str = "minecraft";

    pub fn parse(value: &str) -> Result<Self, ParseError> {
        let (namespace, path) = match value.find(':') {
            Some(separator) if separator != 0 => (&value[..separator], &value[separator + 1..]),
            Some(separator) => (Self::DEFAULT_NAMESPACE, &value[separator + 1..]),
            None => (Self::DEFAULT_NAMESPACE, value),
        };
        if !is_valid_namespace(namespace) || !is_valid_path(path) {
            return Err(ParseError);
        }
        Ok(Self {
            namespace: namespace.to_owned(),
            path: path.to_owned(),
        })
    }

    pub fn from_namespace_and_path(
        namespace: impl Into<String>,
        path: impl Into<String>,
    ) -> Result<Self, ParseError> {
        let namespace = namespace.into();
        let path = path.into();
        if !is_valid_namespace(&namespace) || !is_valid_path(&path) {
            return Err(ParseError);
        }
        Ok(Self { namespace, path })
    }

    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn as_string(&self) -> String {
        format!("{}:{}", self.namespace, self.path)
    }

    pub(crate) fn java_hash_code(&self) -> i32 {
        self.namespace
            .java_string_hash()
            .wrapping_mul(31)
            .wrapping_add(self.path.java_string_hash())
    }
}

trait JavaStringHash {
    fn java_string_hash(&self) -> i32;
}

impl JavaStringHash for str {
    fn java_string_hash(&self) -> i32 {
        self.encode_utf16().fold(0_i32, |hash, unit| {
            hash.wrapping_mul(31).wrapping_add(i32::from(unit))
        })
    }
}

fn is_valid_namespace(value: &str) -> bool {
    value != ".." && value.chars().all(is_valid_namespace_char)
}

fn is_valid_path(value: &str) -> bool {
    value.chars().all(is_valid_path_char)
}

fn is_valid_namespace_char(value: char) -> bool {
    value.is_ascii_lowercase() || value.is_ascii_digit() || matches!(value, '_' | '-' | '.')
}

fn is_valid_path_char(value: char) -> bool {
    is_valid_namespace_char(value) || value == '/'
}
