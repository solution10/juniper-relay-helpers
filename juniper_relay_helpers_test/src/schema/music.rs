use juniper::graphql_object;
use uuid::Uuid;
use juniper_relay_helpers::{RelayConnection, OffsetCursor};
use crate::context::Context;
use crate::schema::Location;

#[derive(Clone)]
pub struct MusicRow {
    pub id: Uuid,
    pub title: String,
}

/// This is an example of a "complex" field resolver type, where there is a dedicated impl
/// block for the field resolver, which also contains a context parameter.
#[derive(Clone, Debug, RelayConnection, Eq, PartialEq)]
#[relay(context = Context, cursor = OffsetCursor)]
pub struct MusicTrack {
    pub id: Uuid,
    pub title: String,
}

impl From<MusicRow> for MusicTrack {
    fn from(row: MusicRow) -> Self {
        Self { id: row.id, title: row.title }
    }
}

#[graphql_object(context = Context)]
impl MusicTrack {
    pub fn id(&self) -> &Uuid {
        &self.id
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    // This lad is the problem - by having the `Context` parameter, the derive macro has to
    // handle it differently.
    pub fn locations_heard(&self, ctx: &Context) -> Vec<Location> {
        let first_location = ctx.locations.first().unwrap();
        vec![
            Location::from(first_location.clone())
        ]
    }
}

pub fn get_music_test_data() -> Vec<MusicRow> {
    vec![
        MusicRow { id: Uuid::new_v4(), title: "Une vie a t'aimer".to_string() },
        MusicRow { id: Uuid::new_v4(), title: "Our drafts collide".to_string() }
    ]
}
