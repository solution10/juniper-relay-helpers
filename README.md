# Juniper Relay GraphQL spec helpers

[![⚒️ Build and test](https://github.com/solution10/graphql-relay-helpers/actions/workflows/branch-test.yml/badge.svg)](https://github.com/solution10/graphql-relay-helpers/actions/workflows/branch-test.yml)
[![crates.io](https://img.shields.io/crates/v/juniper_relay_helpers)](https://crates.io/crates/juniper_relay_helpers)
[![docs.rs](https://docs.rs/juniper_relay_helpers/badge.svg)](https://docs.rs/juniper_relay_helpers)

Crate providing helpers for implementing the [Relay GraphQL spec](https://relay.dev/docs/guides/graphql-server-specification/).

- [Quick Tour](#quick-tour)
- [Documentation](#documentation)
- [Example App](#example-app)
- [Contributing](#contributing)
- [Authors](#authors)

## Quick Tour

Here's a super quick taster of what this crate can do for you!

(These examples are illustrative; see the [Example App](#example-app) and [Documentation](https://docs.rs/juniper_relay_helpers) for the real deal).

```rust
use juniper_relay_helpers::{RelayConnection};

// Add the `RelayConnection` derive macro to your type to get some fancy additional structs:

#[derive(RelayConnection)]
struct User {
    name: String,
}

// UserRelayConnection and UserRelayEdge are generated for you 🎉

// -------------

// Now cursors are a PITA, wouldn't it be nice if we have some helpers for that?

use juniper_relay_helpers::{OffsetCursor};

// Build a cursor that represents a SQL-like offset + limit:
let offset_cursor = OffsetCursor::new(100);

// Build a cursor that's just a raw string from something like Dynamo or an external system:
let string_cursor = StringCursor::new("some-string-cursor");

// These cursors are implemented as GraphQL scalars, so you can use them in your query arguments, and use them in
// response payloads!

// --------------

// Relay identifiers need to be unique across all types. The crate provides a helper struct to generate these, with
// type information encoded into them!

use juniper_relay_helpers::{RelayIdentifier, IdentiferTypeDiscriminator};

// First define your entity types:
#[derive(IdentiferTypeDiscriminator)]
enum MyEntityTypes {
  Character
}

// And then you can generate Relay identifiers:
let identifier = RelayIdentifier::new(123, MyEntityTypes::Character);

// The `RelayIdentifier` type is also implemented as a GraphQL scalar, so you can use it in responses and it is output
// with the `ID` GraphQL type, perfect for Relay!

// --------------

// Tying it all together:
use juniper_relay_helpers::{
  OffsetCursor,
  OffsetCursorProvider,
  IdentiferTypeDiscriminator,
  PageRequest,
  RelayIdentifier
};

#[derive(IdentiferTypeDiscriminator)]
enum MyEntityTypes {
  Character
}

#[derive(GraphQLObject, RelayConnection)]
struct User {
  id: RelayIdentifier<UUID, MyEntityTypes>,
  name: String
}
impl From<UserRow> for User {
  fn from(row: UserRow) -> Self {
    Self {
      id: RelayIdentifier::new(row.id, MyEntityTypes::Character),
      name: row.name,
    }
  }
}

impl QueryRoot {
  fn users(&self, first: Option<i32>, after: Option<OffsetCursor>,  context: &Context) -> FieldResult<Connection<User>> {
    let user_result = fetch_users_from_db(context, first, after.offset);
    
    Ok(UserRelayConnection::new(
      user_result.rows.map(|row| User.from(row)),
      user_result.total_count,
      OffsetCursorProvider::new(),
      Some(PageRequest::new(first, after))
    ))
  }
}

```

## Documentation

The crate aims for a high degree of documentation, so you can find all the details at
[https://docs.rs/juniper_relay_helpers](https://docs.rs/juniper_relay_helpers).

## Example App

There's nothing like seeing things in action! This repo contains an example Axum & Juniper application that uses
this crate to implement a Relay compliant server. Check out [/juniper_relay_helpers_test](/juniper_relay_helpers_test)
a toy GraphQL server that uses this crate and is the basis of the integration tests.

## Contributing

Contributions are welcome! Please see the [Contributing Guide](CONTRIBUTING.md) for more details.

## Authors

- Alex Gisby: (https://github.com/solution10)
