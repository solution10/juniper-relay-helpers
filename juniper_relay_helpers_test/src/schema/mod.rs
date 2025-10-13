pub use crate::schema::character::{
    Character, CharacterRelayConnection, CharacterRelayEdge, CharacterRow,
};
pub use crate::schema::identifiers::EntityType;
pub use crate::schema::location::{Location, LocationRelayConnection, LocationRow};
use juniper::{EmptyMutation, EmptySubscription, FieldResult, RootNode};
use juniper_relay_helpers::{Cursor, CursorProvider, KeyedCursorProvider, OffsetCursor, OffsetCursorProvider, PageInfo, PageRequest, PaginationMetadata, RelayConnection, RelayEdge, RelayIdentifier, StringCursor};

mod character;
mod identifiers;
mod location;

pub use crate::schema::character::get_character_test_data;
pub use crate::schema::location::get_location_test_data;

// ---------- Context -------------

#[derive(Clone)]
pub struct Context {
    pub characters: Vec<CharacterRow>,
    pub locations: Vec<LocationRow>,
}
impl juniper::Context for Context {}

// --------- QueryRoot ------------

pub struct QueryRoot;

#[juniper::graphql_object(context = Context)]
impl QueryRoot {
    /// Queries for all characters in the "database"
    /// This method shows how you can manually build up the resulting structs without using
    /// cursor providers or any of the other fancy stuff.
    async fn characters(ctx: &Context) -> FieldResult<CharacterRelayConnection> {
        Ok(CharacterRelayConnection {
            count: ctx.characters.len() as i32,
            edges: ctx
                .characters
                .iter()
                .enumerate()
                .map(|(idx, row)| {
                    CharacterRelayEdge::new(
                        Character {
                            id: RelayIdentifier::new(row.id, EntityType::Character),
                            name: row.name.clone(),
                        },
                        OffsetCursor {
                            offset: idx as i32,
                            first: Some(10),
                        },
                    )
                })
                .collect(),
            page_info: PageInfo {
                has_next_page: false,
                has_prev_page: false,
                start_cursor: None,
                end_cursor: None,
            },
        })
    }

    /// Queries for all locations in the "database"
    /// This method makes use of cursor providers and the shortcut methods to show how much you can
    /// hand off to the library:
    async fn locations(
        first: Option<i32>,
        after: Option<OffsetCursor>,
        ctx: &Context,
    ) -> FieldResult<LocationRelayConnection> {
        let mut nodes = ctx
            .locations
            .iter()
            .map(|row| Location::from(row.clone()))
            .collect::<Vec<Location>>();

        if let Some(after) = &after {
            nodes = nodes.split_off(after.offset as usize + 1);
        }

        if let Some(first) = first {
            nodes.truncate(first as usize);
        }

        Ok(LocationRelayConnection::new(
            &nodes,
            ctx.locations.len() as i32,
            OffsetCursorProvider::new(),
            Some(PageRequest::new(first, after)),
        ))
    }

    /// Queries for all locations in the "database"
    /// This method makes use of the String cursor provider to show how you can use that one too!
    async fn locations_string_cursor(
        first: Option<i32>,
        after: Option<StringCursor>,
        ctx: &Context,
    ) -> FieldResult<LocationRelayConnection> {
        let mut nodes = ctx
            .locations
            .iter()
            .map(|row| Location::from(row.clone()))
            .collect::<Vec<Location>>();

        let cp = KeyedCursorProvider;
        let pr = PageRequest::new(first, after);

        if let Some(after_cursor) = &pr.after {
            // Find the starting item:
            let idx = nodes.iter().position(|item| {
                let sub_page = PageRequest::new(first, Some(StringCursor::new(after_cursor.clone())));
                let pagination_metadata = PaginationMetadata {
                    total_count: ctx.locations.len() as i32,
                    page_request: Some(sub_page),
                };
                let item_cursor = cp.get_cursor_for_item(&pagination_metadata, 0, item);
                item_cursor.to_encoded_string().eq(after_cursor)
            });

            if let Some(idx) = idx {
                nodes = nodes.split_off(idx + 1);
            }
        }

        if let Some(first) = first {
            nodes.truncate(first as usize);
        }

        Ok(LocationRelayConnection::new(
            &nodes,
            ctx.locations.len() as i32,
            KeyedCursorProvider,
            Some(pr),
        ))
    }
}

// ---------- Schema -------------

pub type Schema = RootNode<QueryRoot, EmptyMutation<Context>, EmptySubscription<Context>>;
