use std::fmt::{Display, Formatter};
use juniper::GraphQLScalar;
use crate::{Cursor, CursorError, CURSOR_SEGMENT_DELIMITER};

/// Built-in cursor type for when the cursor is just a string. Usually useful for things like
/// NoSQL systems that return something opaque to you.
#[derive(Debug, GraphQLScalar, Clone)]
pub struct StringCursor {
    /// The value of the cursor.
    pub value: String,
}

impl StringCursor {
    pub fn new(value: String) -> Self {
        StringCursor { value }
    }
}

impl Cursor for StringCursor {
    type CursorType = StringCursor;

    fn to_raw_string(&self) -> String {
        format!("string{}{}", CURSOR_SEGMENT_DELIMITER, self.value.clone())
    }

    fn new(_raw: &str, parts: Vec<&str>) -> Result<Self::CursorType, CursorError> {
        let raw_parts_value = parts[1].to_string();
        Ok(StringCursor {
            value: raw_parts_value,
        })
    }
}
impl Display for StringCursor {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_raw_string())
    }
}

impl Default for StringCursor {
    fn default() -> Self {
        StringCursor {
            value: "".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Cursor, StringCursor};

    #[test]
    fn test_string_cursor_raw_string() {
        let cursor = StringCursor {
            value: "some-cursor".to_string(),
        };
        assert_eq!(cursor.to_string(), "string||some-cursor");
    }

    #[test]
    fn test_string_cursor_encoded_string() {
        let cursor = StringCursor {
            value: "some-cursor".to_string(),
        };
        assert_eq!(cursor.to_encoded_string(), "c3RyaW5nfHxzb21lLWN1cnNvcg==");
    }

    #[test]
    fn test_string_cursor_from_encoded_string() {
        let cursor = StringCursor::from_encoded_string("c3RyaW5nfHxzb21lLWN1cnNvcg==").unwrap();
        assert_eq!(cursor.value, "some-cursor");
    }

}