//! Library to help with working with the Relay specification, providing Derive macros, structs and
//! traits to help with building a Relay compliant GraphQL server.
//!
//! For use with the Juniper GraphQL framework.
//!
//! # Connections and Edges
//!
//! A main feature of this library is making it easier to generate the required structs for the Connection
//! and Edge types, as well as using them to build responses.
//!
//! ## Code generation of Connections and Edges
//!
//! Define your normal entity struct (the `node` in Relay parlance) and then use the `#[derive(RelayConnection)`
//! macro to automatically generate the `Connection` and `Edge` structs, wired into the `GraphQLObject` etc macros.
//!
//! ```rust
//! use juniper::GraphQLObject;
//! # use juniper_relay_helpers_codegen::{RelayConnection};
//! # use juniper_relay_helpers::PageInfo;
//!
//! #[derive(Debug, GraphQLObject, RelayConnection, Clone, Eq, PartialEq)]
//! struct PlayableCharacter {
//!     pub name: String,
//!     pub theme_song: String,
//! }
//!
//! // Generated structs - written out here to show the full code::
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
//!     cursor: String!
//!     node: PlayableCharacter!
//! }
//! ```
//!
//! **Notes**:
//! - The struct has `RelayConnection` and `RelayEdge` as the suffix to help avoid collisions with your code.
//! - GraphQL types have `Connection` and `Edge` as the suffix to conform to the spec.
//!
//! ## Building Connection responses
//!
//! The generated `RelayConnection` and `RelayEdge` structs have some helper shortcuts on them to make
//! building up responses as terse as possible.
//!
//! `RelayConnection` in particular has a big shortcut you'll want to make usage of.
//!
//! This is an example from the example app in the `/juniper_relay_helpers_test` folder:
//!
//! ```nocompile
//! async fn locations(first: Option<i32>, after: Option<OffsetCursor>, ctx: &Context) -> FieldResult<LocationRelayConnection> {
//!     let mut nodes = ctx.locations
//!         .iter()
//!         .map(|row| Location::from(row.clone()))
//!         .collect::<Vec<Location>>();
//!
//!     if let Some(after) = &after {
//!         nodes = nodes.split_off(after.offset as usize + 1);
//!     }
//!     if let Some(first) = first {
//!         nodes.truncate(first as usize);
//!     }
//!
//!     Ok(
//!         LocationRelayConnection::new(
//!             &nodes,
//!             ctx.locations.len() as i32,
//!             OffsetCursorProvider::new(),
//!             Some(PageRequest::new(first, after))
//!         )
//!     )
//! }
//! ```
//! The `LocationRelayConnection::new` method takes the following arguments:
//!
//! - The nodes to include in the connection
//! - The total count of _all_ results in this query resolver.
//! - A cursor provider to generate cursors for the edges and `PageInfo`.
//! - A `PageRequest` to generate the pagination info from, using the `first` and `after`.
//!
//! With that, it can build up the entire response to the client with correct pagination and cursors.
//!
//! Naturally, you can also manually build up responses yourself and make use of the pagination
//! primitives that the generated code uses and provides.
//!
//! # Pagination
//!
//! The library contains a few helpers to work with pagination.
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
//! You can either manually build this object up yourself or if you use an implementation of `CursorProvider`
//! it can build this information for you.
//!
//! ## Page Request
//!
//! Pagination requests in Relay usually are specified by a ``first`` and ``after`` argument.
//! This library provides a `PageRequest` struct to help with this.
//!
//! ```
//! use juniper_relay_helpers::{PageRequest, StringCursor};
//! #
//! # fn page_request() {
//! let page_request = PageRequest::new(Some(10), Some(StringCursor::new("my-cursor")));
//! # }
//! ```
//!
//! Usage of this is optional for the most part, but if you want to use the `RelayConnection::new` method
//! of building responses, it expects a `PageRequest` to be passed in.
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
//! For a reference implementation, see the `OffsetCursorProvider` struct.
//!
//! For NoSQL use cases, there is also the `KeyedCursorProvider`.
//!
//! **Note**: remember that offset cursors are massively prone to off-by-one errors. The cursor provided
//! to the `after` argument **means** after - if you're using database offsets or memory slices, you need to
//! add `+ 1` to the provided offset to get the _actual_ starting point.
//!
//! See the example app for a more detailed example of how to handle this.
//!
//! # Identifiers
//!
//! Relay requires nodes to have unique identifiers specified by `ID` type. Often you want to encode
//! some useful type information into that identifier. The library contains a simple `RelayIdentifier`
//! struct that can be used to do this.
//!
//! The `RelayIdentifier` takes two arguments - the identifier itself and a type discriminator.
//!
//! ```
//! use std::fmt::{Display, Formatter};
//! use std::str::FromStr;
//! use juniper_relay_helpers::{RelayIdentifier};
//! #
//! use graphql_relay_helpers_codegen::IdentifierTypeDiscriminator;
//!
//! # fn identifiers() {
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
//! This generates a base64 encoded string of the format `type_discriminator::identifier`. It is also
//! implemented as a `GraphQLScalar` for use directly in Juniper, so you can return it directly from
//! your DTO object or field resolver.
//!
//! ## IdentifierTypeDiscriminator
//!
//! To be able to use an `enum` as your identifier discriminator, you need to implement a couple of traits.
//! Or, the easier path, add the `IdentifierTypeDiscriminator` derive macro:
//!
//! ```
//! use juniper_relay_helpers::{IdentifierTypeDiscriminator, RelayIdentifier};
//!
//! #[derive(IdentifierTypeDiscriminator)]
//! enum MyEntityTypes {
//!     CHARACTER,
//!     ENEMY
//! }
//!
//! // This can now be used in RelayIdentifier:
//! let id = RelayIdentifier::new("123".to_string(), MyEntityTypes::CHARACTER);
//! ```
//!
//! The use of `RelayIdentifier` is entirely optional - you can use your own identifiers or the `juniper::ID` type
//! and still make use of the `RelayConnection` derive macro. It's just here if you want it.
//!
//! # Example App
//!
//! You can see the library in action in the example app in `/juniper_relay_helpers_test`.
//!
//! This app is also what's used for the integration tests, so it should be a strong representation of the
//! capabilities of the library.
//!
//! See the README in that folder for more information.
//!

extern crate self as juniper_relay_helpers;

mod connections;
mod cursor_errors;
mod cursor_provider;
mod cursors;
mod edges;
mod identifier;
mod pagination;

// From other crates in the workspace:
pub use juniper_relay_helpers_codegen::{IdentifierTypeDiscriminator, RelayConnection};

// From this crate:
pub use connections::*;
pub use cursor_errors::*;
pub use cursor_provider::*;
pub use cursors::*;
pub use edges::*;
pub use identifier::*;
pub use pagination::*;
