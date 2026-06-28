use base64::prelude::*;
use juniper::{ParseScalarResult, ParseScalarValue, ScalarToken, ScalarValue};
use crate::CursorError;

pub const CURSOR_SEGMENT_DELIMITER: &str = "||";

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
        Self::new(
            decoded_string.as_str(),
            decoded_string.split(CURSOR_SEGMENT_DELIMITER).collect(),
        )
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
            Err(err) => Err(err.to_string().into_boxed_str()),
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
pub fn cursor_from_encoded_string<T>(input: &str) -> Result<T, CursorError>
where
    T: Cursor<CursorType = T>,
{
    let cursor = T::from_encoded_string(input)?;
    Ok(cursor)
}