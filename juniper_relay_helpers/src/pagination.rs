use crate::cursor_errors::CursorError;
use crate::{Cursor, cursor_from_encoded_string};
use juniper::GraphQLObject;

/// Represents the Relay spec pagination object
/// <https://relay.dev/docs/guides/graphql-server-specification/>
///
#[derive(Debug, GraphQLObject, Eq, PartialEq, Clone)]
#[graphql(description = "Pagination information")]
pub struct PageInfo {
    /// Indicates whether there is a page following this current one
    #[graphql(description = "Indicates whether there is a page following this current one")]
    pub has_next_page: bool,

    /// Indicates whether there is a page preceding this one
    #[graphql(description = "Indicates whether there is a page preceding this one")]
    pub has_prev_page: bool,

    /// An opaque cursor that when passed to after: in a query will return the previous page of
    /// results.
    #[graphql(
        description = "An opaque cursor that when passed to after: in a query will return the previous page of results."
    )]
    pub start_cursor: Option<String>,

    /// An opaque cursor that when passed to after: in a query will return the following page of
    /// results.
    #[graphql(
        description = "An opaque cursor that when passed to after: in a query will return the following page of results."
    )]
    pub end_cursor: Option<String>,
}

/// Represents a common Relay pagination request pattern. You'd usually build this from the arguments
/// into the query resolver, and can then pass that into service calls etc.
///
/// Many query resolvers take the form:
///
/// ```graphql
///  query {
///      hairstyles(first: 10, after: "b2Zmc2V0OjE6MTA=") {
///          name
///          available_colors
///     }
///  }
/// ```
///
/// This struct can be used to represent the first and after arguments. It is also a GraphQLObject itself,
/// which means it can be used in the schema directly.
///
#[derive(Debug, GraphQLObject, Eq, PartialEq, Clone)]
#[graphql(description = "Page request")]
pub struct PageRequest {
    /// The number of items to return.
    #[graphql(description = "The number of items to return.")]
    pub first: Option<i32>,

    /// A cursor to use as the pointer to the start of the page.
    #[graphql(description = "A cursor to use as the pointer to the start of the page.")]
    pub after: Option<String>,
}

impl PageRequest {
    /// Helper method to build from the component parts from a query resolver
    pub fn new(first: Option<i32>, after: Option<impl Cursor>) -> Self {
        PageRequest {
            first,
            after: after.map(|after| after.to_encoded_string()),
        }
    }

    /// Parses the `after` portion of the PageRequest into the appropriate cursor type.
    /// Will return `None` if the `Option` is empty, and returns wrapped in a `Result` in case the
    /// decoding of the cursor fails.
    pub fn parsed_cursor<T>(&self) -> Result<Option<T>, CursorError>
    where
        T: Cursor<CursorType = T>,
    {
        if self.after.is_none() {
            return Ok(None);
        }
        let decoded_cursor = cursor_from_encoded_string(self.after.as_ref().unwrap())?;
        Ok(Some(decoded_cursor))
    }
}

#[cfg(test)]
mod tests {
    use crate::{OffsetCursor, PageRequest, StringCursor};

    #[test]
    fn test_new() {
        let pr = PageRequest::new(
            Some(10),
            Some(StringCursor::new("some-string-cursor".to_string())),
        );
        assert_eq!(pr.first, Some(10));
        assert_eq!(
            pr.after,
            Some("c3RyaW5nfHxzb21lLXN0cmluZy1jdXJzb3I=".to_string())
        );
    }

    #[test]
    fn test_decoding_cursor_from_page_request() {
        let request = PageRequest {
            first: Some(10),
            after: Some("b2Zmc2V0fHwxfHwxMA==".to_string()),
        };
        let decoded_cursor = request.parsed_cursor::<OffsetCursor>().unwrap();
        assert_eq!(decoded_cursor.unwrap().offset, 1);
    }
}
