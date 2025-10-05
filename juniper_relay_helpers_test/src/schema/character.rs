use std::str::FromStr;
use juniper::GraphQLObject;
use uuid::Uuid;
use juniper_relay_helpers::{RelayConnection, RelayIdentifier};

use crate::schema::identifiers::{EntityType};

/// "Database" row for a character.
#[derive(Clone)]
pub struct CharacterRow {
    pub id: Uuid,
    pub name: String,
}

/// GraphQL type for a character.
#[derive(GraphQLObject, RelayConnection, Debug, Eq, PartialEq, Clone)]
pub struct Character {
    pub id: RelayIdentifier<Uuid, EntityType>,
    pub name: String,
}


// ----------- Test data ------------------

pub fn get_character_test_data() -> Vec<CharacterRow> {
    vec![
        CharacterRow {
            id: Uuid::from_str("a39e7f9b-4237-4e8f-b437-e4559cdd3482").unwrap(),
            name: "Lune".to_string()
        },
        CharacterRow {
            id: Uuid::from_str("6e548bf7-acbb-4b65-9c99-d7594b653ebb").unwrap(),
            name: "Sciel".to_string()
        },
        CharacterRow {
            id: Uuid::from_str("b911af2c-5526-49fa-bd45-1af862a7220f").unwrap(),
            name: "Maelle".to_string()
        },
        CharacterRow {
            id: Uuid::from_str("a2e7fb70-a9f8-4e60-b571-9d0afdc4b468").unwrap(),
            name: "Gustave".to_string()
        },
        CharacterRow {
            id: Uuid::from_str("f461e8d6-6400-46ca-9aee-a129d14fe83c").unwrap(),
            name: "Monoco".to_string()
        },
    ]
}
