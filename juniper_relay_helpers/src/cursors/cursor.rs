use base64::prelude::*;
use juniper::{
    DefaultScalarValue, FromInputValue, GraphQLType, GraphQLValue, GraphQLValueAsync,
    ParseScalarResult, ParseScalarValue, ScalarToken, ScalarValue,
    macros::reflect::{BaseSubTypes, BaseType, WrappedType},
    marker::IsOutputType,
};
use crate::CursorError;

pub const CURSOR_SEGMENT_DELIMITER: &str = "||";

/// Non-generic cursor interface: serialization, deserialization, and the scalar helper methods
/// used to wire a cursor type into `#[derive(GraphQLScalar)]`.
///
/// Implement this trait on your cursor struct, then add an empty `impl<S: ScalarValue + Send + Sync> Cursor<S> for MyCursor {}`
/// to make it usable as a typed cursor in connections.
///
/// ```nocompile
/// #[derive(Debug, GraphQLScalar)]
/// #[graphql(
///     name = "MyCursor",
///     to_output_with = Self::to_output,
///     from_input_with = Self::from_input
/// )]
/// struct MyCursor {}
/// impl CursorBase for MyCursor { ... }
/// impl<S: ScalarValue + Send + Sync> Cursor<S> for MyCursor {}
/// ```
///
pub trait CursorBase: Clone + Sized {
    /// Concrete type of the returned cursor. Usually the thing that implements the trait.
    type CursorType: CursorBase;

    /// Serialize the cursor into a string ready to be base64 encoded.
    fn to_raw_string(&self) -> String;

    /// Constructor that given the raw string, and a vector of parts (the delimiter-separated segments)
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

    // ------------- GraphQLScalar helper implementations --------------

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

/// Marker trait that combines [`CursorBase`] with all Juniper GraphQL scalar output traits
/// for a given scalar value type `S`.
///
/// Implement this as an empty impl on any cursor type that implements both `CursorBase` and
/// `#[derive(GraphQLScalar)]`:
///
/// ```nocompile
/// impl<S: ScalarValue + Send + Sync> Cursor<S> for MyCursor {}
/// ```
///
pub trait Cursor<S: ScalarValue + Send + Sync = DefaultScalarValue>:
    CursorBase
    + Send
    + Sync
    + FromInputValue<S>
    + GraphQLValue<S, TypeInfo = (), Context = ()>
    + GraphQLType<S>
    + IsOutputType<S>
    + GraphQLValueAsync<S>
    + BaseType<S>
    + BaseSubTypes<S>
    + WrappedType<S>
{
}

/// Decodes a cursor from a base64 encoded string into the correct concrete instance type.
/// Use the Turbofish `::<>()` syntax to tell the method what that correct type is.
///
/// For instance, to parse out an Offset cursor:
///
/// ```rust
/// use juniper_relay_helpers::{cursor_from_encoded_string, OffsetCursor};
///
/// let decoded_cursor = cursor_from_encoded_string::<OffsetCursor>("b2Zmc2V0OjE6MTA=");
/// ```
///
/// `decoded_cursor` will be a `Result<OffsetCursor, CursorError>` in case the decoding fails.
///
pub fn cursor_from_encoded_string<T>(input: &str) -> Result<T, CursorError>
where
    T: CursorBase<CursorType = T>,
{
    T::from_encoded_string(input)
}
