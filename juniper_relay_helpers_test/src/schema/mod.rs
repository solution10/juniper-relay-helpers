pub use crate::context::Context;
pub use crate::schema::character::{
    Character, CharacterRelayConnection, CharacterRelayEdge, CharacterRow,
};
pub use crate::schema::identifiers::EntityType;
pub use crate::schema::location::{Location, LocationRelayConnection, LocationRow};
pub use crate::schema::music::{MusicRow, MusicTrack};
use juniper::{EmptyMutation, EmptySubscription, FieldResult, RootNode};
use juniper_relay_helpers::{
    OffsetCursor, OffsetCursorProvider, PageInfo,
    PageRequest, RelayConnection, RelayEdge, RelayIdentifier,
};

mod character;
mod identifiers;
mod location;
mod music;

pub use crate::schema::character::get_character_test_data;
pub use crate::schema::location::get_location_test_data;
pub use crate::schema::music::get_music_test_data;


// --------- QueryRoot ------------

pub struct QueryRoot;

#[juniper::graphql_object(context = Context, scalar = S: juniper::ScalarValue + Send + Sync)]
impl QueryRoot {
    /// Queries for all characters in the "database"
    /// This method shows how you can manually build up the resulting structs without using
    /// cursor providers or any of the other fancy stuff.
    async fn characters(ctx: &Context) -> FieldResult<CharacterRelayConnection> {
        Ok(CharacterRelayConnection {
            count: Some(ctx.characters.len() as i32),
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
                        OffsetCursor::new(idx as i32),
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
            Some(ctx.locations.len() as i32),
            OffsetCursorProvider::new(),
            Some(PageRequest::new(first, after, None)),
        ))
    }

    /// Queries for all locations using an offset cursor from a numeric page arg.
    async fn locations_paged(
        first: Option<i32>,
        page: Option<i32>,
        ctx: &Context,
    ) -> FieldResult<LocationRelayConnection> {
        let after = page.map(OffsetCursor::new);
        let mut nodes = ctx
            .locations
            .iter()
            .map(|row| Location::from(row.clone()))
            .collect::<Vec<Location>>();

        if let Some(ref c) = after {
            nodes = nodes.split_off(c.offset as usize + 1);
        }
        if let Some(first) = first {
            nodes.truncate(first as usize);
        }

        Ok(LocationRelayConnection::new(
            &nodes,
            Some(ctx.locations.len() as i32),
            OffsetCursorProvider::new(),
            Some(PageRequest::new(first, after, None)),
        ))
    }

    async fn music(ctx: &Context) -> FieldResult<Vec<MusicTrack>> {
        Ok(ctx.music.iter().map(|r| r.clone().into()).collect())
    }
}

// ---------- Schema -------------

pub type Schema = RootNode<QueryRoot, EmptyMutation<Context>, EmptySubscription<Context>>;
