//! Library to help with working with the Relay specification. Contains derive macros for generating
//! connection and edge structs, as well as structs for the Pagination information and handling
//! of Relay cursors and Identifiers.
//!
//! # Pagination helpers
//!
//! The library contains a few helper functions to help with pagination.
//!
//! ## PageInfo
//!
//! The PageInfo struct is a ready to use GraphQLObject that conforms to the Relay spec. This struct
//! is added to your Connection types generated from `RelayConnection`.
//!
//! It'll add the type:
//!
//! ```graphql
//! type PageInfo {
//!     hasNextPage: Boolean!
//!     hasPreviousPage: Boolean!
//!     startCursor: String
//!     endCursor: String
//! }
//! ```
//!
//! ## Cursors
//!
//! Relay requires edges and pagination info to contain opaque strings called "cursors".
//! This library provides a few built-in cursors, but you can also implement your own.
//!
//! The most simple cursor is the OffsetCursor, which is just an offset and a limit, similar to
//! SQL LIMIT and OFFSET.
//!
//! ```
//! # use juniper_relay_helpers::{cursor_from_encoded_string, Cursor, OffsetCursor};
//! #
//! # fn cursors() {
//! let cursor = OffsetCursor { offset: 1, first: 10 };
//!
//! // Encode the cursor into a string of format "offset:1:10"
//! let cursor_string = cursor.to_raw_string();
//!
//! // Encode the raw string into a base64 encoded string
//! let encoded_string = cursor.to_encoded_string();
//!
//! // You can also decode the cursor from the base64 encoded string
//! let decoded_cursor = OffsetCursor::from_encoded_string(&encoded_string).unwrap();
//! let decoded_cursor_turbo = cursor_from_encoded_string::<OffsetCursor>(&encoded_string).unwrap();
//! #
//! # }
//! ```
//!
//! Implementing your own cursor is as simple as implementing the `Cursor` trait.
//!
//! ## Cursor providers
//!
//! Relay requires edges and pagination info to contain cursors, which can be annoying to generate
//! and add to the connection.
//!
//! `CursorProvider` is a trait that allows you to easily generate cursors for each of the items
//! in the result set.
//!
//! todo().
//!
//! # Identifiers
//!
//! Relay requires nodes to have unique identifiers specified by `ID` type. Often you want to encode
//! some useful type information into that identifier. The library contains a simple `RelayIdentifier`
//! struct that can be used to do this.
//!
//! ```
//! use std::fmt::{Display, Formatter};
//! use std::str::FromStr;
//! use juniper_relay_helpers::{RelayIdentifier};
//! #
//! use graphql_relay_helpers_codegen::IdentifierTypeDiscriminator;
//!
//! fn identifiers() {
//! #[derive(IdentifierTypeDiscriminator)]
//! enum MyTypes {
//!     Character,
//!     Enemy
//! }
//!
//! let id = RelayIdentifier::new("123".to_string(), MyTypes::Character);
//! # }
//!```
//!
//! This generates a base64 encoded string of the format `type_delimiter::identifier`. It is also
//! implemented as a `GraphQLScalar` for use directly in Juniper, so you can return it directly from
//! your DTO object or field resolver.
//!
//! **Note** that you need to implement `Display` and `FromStr` yourself for the type discriminator
//! enum. You can either do this manually, OR use the `IdentifierTypeDiscriminator` derive macro.
//!
//! # Code Generation
//!
//! ## RelayConnection
//!
//! Derive macro to generate the connection and edge structs for a given object.
//!
//! The object that this is applied to must also `#[derive(juniper::GraphQLObject)]` otherwise you'll
//! get a compilation error.
//!
//! Given the following struct:
//!
//! ```
//! use juniper::GraphQLObject;
//! # use juniper_relay_helpers_codegen::{RelayConnection};
//! # use juniper_relay_helpers::PageInfo;
//!
//! #[derive(Debug, GraphQLObject, RelayConnection, Clone, Eq, PartialEq)]
//! struct PlayableCharacter {
//!     pub name: String,
//!     pub theme_song: String,
//! }
//! ```
//!
//! The `RelayConnection` macro will generate two additional structs:
//!
//! ```rust
//! # use juniper::GraphQLObject;
//! # use juniper_relay_helpers::PageInfo;
//! #
//! # #[derive(GraphQLObject)]
//! # struct PlayableCharacter {
//! #     pub name: String,
//! #     pub theme_song: String,
//! # }
//! #
//! // Generated structs:
//! #[derive(GraphQLObject)]
//! struct PlayableCharacterRelayConnection {
//!     count: i32,
//!     edges: Vec<PlayableCharacterRelayEdge>,
//!     page_info: PageInfo
//! }
//!
//! #[derive(GraphQLObject)]
//! struct PlayableCharacterRelayEdge {
//!     cursor: String,
//!     node: PlayableCharacter,
//! }
//!
//! ```
//!
//! With the following types generated for the GraphQL schema:
//!
//! ```graphql
//! type PlayableCharacterConnection {
//!     count: Int!
//!     edges: [PlayableCharacterEdge]!
//!     pageInfo: PageInfo!
//! }
//!
//! type PlayableCharacterEdge {
//! 	cursor: String!
//! 	node: PlayableCharacter!
//! }
//! ```
//!
//!

extern crate self as juniper_relay_helpers;

mod pagination;
mod cursors;
mod cursor_errors;
mod cursor_provider;
mod identifier;
mod connections;
mod edges;

// From other crates in the workspace:
pub use juniper_relay_helpers_codegen::{RelayConnection, IdentifierTypeDiscriminator};

// From this crate:
pub use pagination::*;
pub use cursors::*;
pub use cursor_errors::*;
pub use identifier::*;
pub use connections::*;
pub use edges::*;
pub use cursor_provider::*;
