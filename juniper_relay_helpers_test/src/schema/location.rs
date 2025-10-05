use juniper::GraphQLObject;
use juniper_relay_helpers::{RelayConnection, RelayIdentifier};

use crate::schema::identifiers::{EntityType};

/// "Database" row for a location.
#[derive(Clone)]
pub struct LocationRow {
    pub id: String,
    pub name: String,
}

/// GraphQL type for a character.
#[derive(GraphQLObject, RelayConnection, Debug, Eq, PartialEq, Clone)]
pub struct Location {
    pub id: RelayIdentifier<String, EntityType>,
    pub name: String,
}

/// Implement From to give a cleaner experience;
impl From<LocationRow> for Location {
    fn from(row: LocationRow) -> Self {
        Location {
            id: RelayIdentifier::new(row.id, EntityType::Location),
            name: row.name
        }
    }   
}

// ----------- Test data ------------------

pub fn get_location_test_data() -> Vec<LocationRow> {
    vec![
        LocationRow {
            id: "lumiere".to_string(),
            name: "Lumiére".to_string()
        },
        LocationRow {
            id: "esquies-nest".to_string(),
            name: "Esquie's Nest".to_string()
        },
        LocationRow {
            id: "monocos-station".to_string(),
            name: "Monoco's Station".to_string()
        },
    ]
}
