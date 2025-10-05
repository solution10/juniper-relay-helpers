use std::fmt::{Display, Formatter};
use base64::prelude::*;
use juniper::{GraphQLScalar, ParseScalarResult, ParseScalarValue, ScalarToken, ScalarValue};
use crate::cursor_errors::CursorError;

/// Cursor struct that builds into an opaque string.
/// Cursors are present both in the edges and in the PageInfo within the Connection.
///
/// You can implement this trait for your own cursor type if it's not covered by this library.
/// You can also use the built-in Cursors:
///     - OffsetCursor
///     - StringCursor
///
/// This trait implements the common methods needed to be considered a `GraphQlScalar`
/// which means you can add the following to your struct and it will work
/// out of the box:
///
/// ```nocompile
/// #[derive(Debug, GraphQLScalar)]
/// #[graphql(
///     name = "MyCursor",
///     to_output_with = Self::to_output,
///     from_input_with = Self::from_input
/// )]
/// struct MyCursor {}
/// impl Cursor for MyCursor { ... }
/// ```
///
pub trait Cursor {
    /// Concrete type of the returned cursor. Usually the thing that implements the trait.
    type CursorType;

    /// Serialize the cursor into a string ready to be base64 encoded.
    fn to_raw_string(&self) -> String;

    /// Constructor that given the raw string, and a vector of parts (the colon separated segments)
    /// will return a Result of the CursorType. Return a CursorError if the decoding fails.
    fn new(raw: &str, parts: Vec<&str>) -> Result<Self::CursorType, CursorError>;

    /// Builds the CursorType from a base64 encoded string.
    /// Returns a CursorError if the decoding fails.
    fn from_encoded_string(input: &str) -> Result<Self::CursorType, CursorError> {
        let decoded = BASE64_URL_SAFE.decode(input)?;
        let decoded_string = String::from_utf8(decoded)?;
        Self::new(decoded_string.as_str(), decoded_string.split(':').collect())
    }

    /// Builds the base64 encoded variant of the cursor.
    /// Uses the url safe alphabet.
    fn to_encoded_string(&self) -> String {
        BASE64_URL_SAFE.encode(self.to_raw_string().as_bytes())
    }

    // ------------- GraphQLScalar implementations --------------

    fn to_output(&self) -> String {
        self.to_encoded_string()
    }

    fn from_input(input: &str) -> Result<Self::CursorType, Box<str>> {
        let res = Self::from_encoded_string(input);
        match res {
            Ok(cursor) => Ok(cursor),
            Err(err) => Err(err.to_string().into_boxed_str())
        }
    }

    fn parse_token<S: ScalarValue>(value: ScalarToken<'_>) -> ParseScalarResult<S> {
        <String as ParseScalarValue<S>>::from_str(value)
    }
}

/// Decodes a cursor from a base64 encoded string into the correct concrete instance type.
/// Use the Turbofish `::<>()` syntax to tell the method what that correct type is.
///
/// For instance, to parse out an Offset cursor:
///
/// ```rust
/// use graphql_relay_helpers::{cursor_from_encoded_string, OffsetCursor};
///
/// let decoded_cursor = cursor_from_encoded_string::<OffsetCursor>("b2Zmc2V0OjE6MTA=");
/// ```
///
/// `decoded_cursor` will be a `Result<OffsetCursor, CursorError>` in case the decoding fails.
///
pub fn cursor_from_encoded_string<T>(input: &str) -> Result<T, CursorError> where T: Cursor<CursorType = T> {
    let cursor = T::from_encoded_string(input)?;
    Ok(cursor)
}

/// A simple offset-based cursor.
#[derive(Debug, GraphQLScalar)]
#[graphql(
    name = "OffsetCursor",
    to_output_with = Self::to_output,
    from_input_with = Self::from_input
)]
pub struct OffsetCursor {
    /// The offset of the cursor (how many items to skip).
    pub offset: i32,

    /// The number of items to return.
    pub first: Option<i32>,
}

impl Cursor for OffsetCursor {
    type CursorType = OffsetCursor;

    fn to_raw_string(&self) -> String {
        if let Some(first) = self.first {
            format!("offset:{}:{}", self.offset, first)
        } else {
            format!("offset:{}", self.offset)
        }
    }

    fn new(_raw: &str, parts: Vec<&str>) -> Result<OffsetCursor, CursorError> {
        if parts.len() != 2 && parts.len() != 3 {
            return Err(CursorError::InvalidCursor);
        }

        // Offset is always defined
        let offset = parts[1].parse::<i32>()
            .unwrap_or(0);

        // First is optional and can be missing
        let first: Option<i32> = if parts.len() == 2{
            None
        } else {
            match parts[2].parse::<i32>() {
                Ok(f) => Some(f),
                Err(_) => None,
            }
        };

        Ok(OffsetCursor { offset, first })
    }
}

impl Display for OffsetCursor {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_raw_string())
    }
}

impl Default for OffsetCursor {
    fn default() -> Self {
        OffsetCursor { offset: 0, first: None }
    }
}

/// Built-in cursor type for when the cursor is just a string. Usually useful for things like
/// NoSQL systems that return something opaque to you.
#[derive(Debug, GraphQLScalar)]
pub struct StringCursor {
    /// The value of the cursor.
    value: String,
}

impl Cursor for StringCursor {
    type CursorType = StringCursor;

    fn to_raw_string(&self) -> String {
        format!("string:{}", self.value.clone())
    }

    fn new(_raw: &str, parts: Vec<&str>) -> Result<Self::CursorType, CursorError> {
        let raw_parts_value= parts[1].to_string();
        Ok(StringCursor { value: raw_parts_value })
    }
}
impl Display for StringCursor {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_raw_string())
    }
}

impl Default for StringCursor {
    fn default() -> Self {
        StringCursor { value: "".to_string() }
    }
}


#[cfg(test)]
mod tests {
    use crate::{Cursor, StringCursor};
    use crate::cursors::OffsetCursor;

    #[test]
    fn test_offset_cursor_raw_string() {
        let cursor = OffsetCursor { offset: 1, first: Some(10) };
        assert_eq!(cursor.to_string(), "offset:1:10");
    }

    #[test]
    fn test_offset_cursor_encoded_string() {
        let cursor = OffsetCursor { offset: 1, first: Some(10) };
        assert_eq!(cursor.to_encoded_string(), "b2Zmc2V0OjE6MTA=");
    }

    #[test]
    fn test_offset_cursor_from_encoded_string() {
        let cursor = OffsetCursor::from_encoded_string("b2Zmc2V0OjE6MTA=").unwrap();
        assert_eq!(cursor.offset, 1);
        assert_eq!(cursor.first, Some(10));
    }

    #[test]
    fn test_string_cursor_raw_string() {
        let cursor = StringCursor { value: "some-cursor".to_string() };
        assert_eq!(cursor.to_string(), "string:some-cursor");
    }

    #[test]
    fn test_string_cursor_encoded_string() {
        let cursor = StringCursor { value: "some-cursor".to_string() };
        assert_eq!(cursor.to_encoded_string(), "c3RyaW5nOnNvbWUtY3Vyc29y");
    }

    #[test]
    fn test_string_cursor_from_encoded_string() {
        let cursor = StringCursor::from_encoded_string("c3RyaW5nOnNvbWUtY3Vyc29y").unwrap();
        assert_eq!(cursor.value, "some-cursor");
    }
}