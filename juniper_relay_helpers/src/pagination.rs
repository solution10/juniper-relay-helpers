use crate::{Cursor, CursorBase};
use juniper::{
    ArcStr, Arguments, BoxFuture, ExecutionResult, Executor, GraphQLType, GraphQLValue,
    GraphQLValueAsync, ScalarValue,
    executor::Registry,
    macros::reflect::{BaseSubTypes, BaseType, WrappedType},
    marker::IsOutputType,
    meta::{Field, MetaType},
};

/// Represents the Relay spec pagination object
/// <https://relay.dev/docs/guides/graphql-server-specification/>
///
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct PageInfo<CursorType> {
    /// Indicates whether there is a page following this current one
    pub has_next_page: bool,

    /// Indicates whether there is a page preceding this one
    pub has_prev_page: bool,

    /// An opaque cursor that when passed to after: in a query will return the previous page of results.
    pub start_cursor: Option<CursorType>,

    /// An opaque cursor that when passed to after: in a query will return the following page of results.
    pub end_cursor: Option<CursorType>,
}

// --- Reflection traits ---

impl<S: ScalarValue, CursorType> BaseType<S> for PageInfo<CursorType> {
    const NAME: juniper::macros::reflect::Type = "PageInfo";
}

impl<S: ScalarValue, CursorType> WrappedType<S> for PageInfo<CursorType> {
    const VALUE: juniper::macros::reflect::WrappedValue = 1;
}

impl<S: ScalarValue, CursorType> BaseSubTypes<S> for PageInfo<CursorType> {
    const NAMES: juniper::macros::reflect::Types = &[<Self as BaseType<S>>::NAME];
}

// --- GraphQL output traits ---

impl<S, CursorType> IsOutputType<S> for PageInfo<CursorType>
where
    S: ScalarValue + Send + Sync,
    CursorType: Cursor<S>,
{
}

impl<S, CursorType> GraphQLType<S> for PageInfo<CursorType>
where
    S: ScalarValue + Send + Sync,
    CursorType: Cursor<S>,
{
    fn name(_: &()) -> Option<ArcStr> {
        Some(juniper::arcstr::literal!("PageInfo"))
    }

    fn meta(info: &(), registry: &mut Registry<S>) -> MetaType<S> {
        let fields: &[Field<S>] = &[
            registry
                .field::<bool>("hasNextPage", info)
                .description("Indicates whether there is a page following this current one"),
            registry
                .field::<bool>("hasPrevPage", info)
                .description("Indicates whether there is a page preceding this one"),
            registry
                .field::<Option<CursorType>>("startCursor", info)
                .description("An opaque cursor; pass to `after:` to get the previous page."),
            registry
                .field::<Option<CursorType>>("endCursor", info)
                .description("An opaque cursor; pass to `after:` to get the next page."),
        ];
        registry
            .build_object_type::<Self>(info, fields)
            .description("Pagination information")
            .into_meta()
    }
}

impl<S, CursorType> GraphQLValue<S> for PageInfo<CursorType>
where
    S: ScalarValue + Send + Sync,
    CursorType: Cursor<S>,
{
    type TypeInfo = ();
    type Context = ();

    fn type_name(&self, _: &()) -> Option<ArcStr> {
        Some(juniper::arcstr::literal!("PageInfo"))
    }

    fn resolve_field(
        &self,
        info: &(),
        field_name: &str,
        _: &Arguments<S>,
        executor: &Executor<(), S>,
    ) -> ExecutionResult<S> {
        match field_name {
            "hasNextPage" => executor.resolve(info, &self.has_next_page),
            "hasPrevPage" => executor.resolve(info, &self.has_prev_page),
            "startCursor" => executor.resolve(info, &self.start_cursor),
            "endCursor" => executor.resolve(info, &self.end_cursor),
            _ => panic!("Field '{}' not found on type 'PageInfo'", field_name),
        }
    }
}

impl<S, CursorType> GraphQLValueAsync<S> for PageInfo<CursorType>
where
    S: ScalarValue + Send + Sync,
    CursorType: Cursor<S> + Sync,
    Self: Sync,
{
    fn resolve_field_async<'a>(
        &'a self,
        info: &'a (),
        field_name: &'a str,
        args: &'a Arguments<S>,
        executor: &'a Executor<(), S>,
    ) -> BoxFuture<'a, ExecutionResult<S>> {
        Box::pin(async move { self.resolve_field(info, field_name, args, executor) })
    }
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
pub struct PageRequest<CursorType> where CursorType: CursorBase {
    /// The number of items to return.
    pub first: Option<i32>,

    /// A cursor to use as the pointer to use as the end of the page
    pub before: Option<CursorType>,

    /// A cursor to use as the pointer to the start of the page.
    pub after: Option<CursorType>,
}

impl<CursorT> PageRequest<CursorT> where CursorT: CursorBase {
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
