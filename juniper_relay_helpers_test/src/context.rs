// ---------- Context -------------

use crate::schema::{CharacterRow, LocationRow, MusicRow};

#[derive(Clone)]
pub struct Context {
    pub characters: Vec<CharacterRow>,
    pub locations: Vec<LocationRow>,
    pub music: Vec<MusicRow>
}
impl juniper::Context for Context {}
