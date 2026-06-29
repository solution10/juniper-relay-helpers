use crate::{Cursor};
use juniper::{GraphQLObject};

/// Represents the Relay spec pagination object
/// <https://relay.dev/docs/guides/graphql-server-specification/>
///
#[derive(Debug, GraphQLObject, Eq, PartialEq, Clone)]
#[graphql(description = "Pagination information")]
pub struct PageInfo<CursorType> where CursorType: Cursor {
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
    pub start_cursor: Option<CursorType>,

    /// An opaque cursor that when passed to after: in a query will return the following page of
    /// results.
    #[graphql(
        description = "An opaque cursor that when passed to after: in a query will return the following page of results."
    )]
    pub end_cursor: Option<CursorType>,
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
/// This struct can be used to represent the first and after arguments.
///
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct PageRequest<CursorType> where CursorType: Cursor {
    /// The number of items to return.
    pub first: Option<i32>,

    /// A cursor to use as the pointer to use as the end of the page
    pub before: Option<CursorType>,

    /// A cursor to use as the pointer to the start of the page.
    pub after: Option<CursorType>,
}

impl<CursorT> PageRequest<CursorT> where CursorT: Cursor {
    /// Helper method to build from the component parts from a query resolver
    pub fn new(first: Option<i32>, after: Option<CursorT>, before: Option<CursorT>) -> Self {
        PageRequest {
            first,
            before,
            after,
        }
    }

    /// Checks after, and then before, to return the current cursor we're working with.
    pub fn current_cursor(&self) -> Option<CursorT> {
        match &self.after {
            Some(after) => Some(after.clone()),
            None => self.before.clone(),
        }
    }
}
