//! Test crate for the graphql_relay_helpers library.
//!
//! Contains a simple, in-memory only GraphQL server that showcases how the library can be used
//! within a real application.
//!
//! Application is also used for integration tests.
//!

use std::sync::Arc;
use axum::extract::State;
use axum::response::{Html, IntoResponse};
use axum::Router;
use axum::routing::{get, post};
use juniper::{EmptyMutation, EmptySubscription};
use juniper::http::graphiql;
use juniper_axum::extract::JuniperRequest;
use juniper_axum::response::JuniperResponse;
use tracing::info;
use crate::schema::{get_character_test_data, get_location_test_data, Context, QueryRoot, Schema};

mod generated_schema;
mod schema;

#[derive(Clone)]
struct AppContext {
    schema: Arc<Schema>,
    graphql_context: Context,
}

fn build_app() -> Router {
    // Build the schema:
    let schema = Arc::new(Schema::new(QueryRoot, EmptyMutation::new(), EmptySubscription::new()));

    // Build the app context:
    let ctx = AppContext {
        schema,
        graphql_context: Context {
            characters: get_character_test_data(),
            locations: get_location_test_data(),
        }
    };

    // Build the server:
    Router::new()
        .route("/", get(|| async { "Ok" }))
        .route("/gui", get(graphiql_handler))
        .route("/graphql", post(graphql_handler))
        .with_state(ctx)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get some logging:
    tracing_subscriber::fmt()
        .pretty()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let app = build_app();

    info!("Starting server on http://localhost:3030");
    info!("GraphiQL IDE available at http://localhost:3030/gui");

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3030").await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn graphiql_handler() -> impl IntoResponse {
    Html(graphiql::graphiql_source("/graphql", None))
}

async fn graphql_handler(
    State(ctx): State<AppContext>,
    JuniperRequest(request): JuniperRequest,
) -> impl IntoResponse {
    JuniperResponse(request.execute(&*ctx.schema, &ctx.graphql_context).await)
}

#[cfg(test)]
mod integration_tests {
    use axum_test::TestServer;
    use crate::build_app;

    #[tokio::test]
    async fn test_server_starts() {
        let app = build_app();
        let server = TestServer::new(app).unwrap();
        let response = server.get("/").await;
        response.assert_status_ok();
    }

    const ALL_CHARACTERS_QUERY: &str = r"
            query Characters {
                characters {
                    count
                    edges {
                        node {
                            id
                            name
                        }
                        cursor
                    }
                    pageInfo {
                        startCursor
                        endCursor
                        hasNextPage
                        hasPrevPage
                    }
                }
            }";

    const ALL_LOCATIONS_QUERY: &str = r"
            query Locations {
                locations {
                    count
                    edges {
                        node {
                            id
                            name
                        }
                        cursor
                    }
                    pageInfo {
                        startCursor
                        endCursor
                        hasNextPage
                        hasPrevPage
                    }
                }
            }";

    mod connection_tests {
        use axum_test::expect_json::__private::serde_json;
        use axum_test::expect_json::__private::serde_json::json;
        use axum_test::expect_json;
        use axum_test::TestServer;
        use serde::{Deserialize, Serialize};
        use juniper_relay_helpers::RelayIdentifier;
        use crate::build_app;
        use crate::integration_tests::{ALL_CHARACTERS_QUERY, ALL_LOCATIONS_QUERY};
        use crate::schema::{get_character_test_data, get_location_test_data, EntityType};

        #[derive(Serialize, Deserialize, Debug, Clone)]
        struct GraphQLPayload {
            query: String,
            variables: Option<serde_json::Value>,
        }

        #[tokio::test]
        async fn test_character_connections() {
            let app = build_app();
            let server = TestServer::new(app).unwrap();

            let response = server
                .post("/graphql")
                .json(&GraphQLPayload {
                    query: ALL_CHARACTERS_QUERY.to_string(),
                    variables: Some(serde_json::Value::Object(serde_json::Map::new()))
                })
                .await;

            response.assert_status_ok();

            // Verify the shape of the response
            let character_test_data = get_character_test_data();
            response.assert_json(&json!({
                "data": expect_json::object().contains(json!({
                    "characters": expect_json::object().contains(json!({
                        "count": character_test_data.len(),
                        "edges": expect_json::array()
                            .len(character_test_data.len())
                            .all(json!({
                                "node": expect_json::object().contains(json!({
                                    "id": expect_json::string(),
                                    "name": expect_json::string(),
                                })),
                                "cursor": expect_json::string(),
                            })),
                        "pageInfo": expect_json::object().contains(json!({
                            "hasNextPage": false,
                            "hasPrevPage": false
                        }))
                    }))
                }))
            }))
        }

        #[tokio::test]
        async fn test_location_connections() {
            let app = build_app();
            let server = TestServer::new(app).unwrap();

            let response = server
                .post("/graphql")
                .json(&GraphQLPayload {
                    query: ALL_LOCATIONS_QUERY.to_string(),
                    variables: Some(serde_json::Value::Object(serde_json::Map::new()))
                })
                .await;

            response.assert_status_ok();

            // Verify the shape of the response
            let location_test_data = get_location_test_data();
            response.assert_json(&json!({
                "data": expect_json::object().contains(json!({
                    "locations": expect_json::object().contains(json!({
                        "count": location_test_data.len(),
                        "edges": expect_json::array()
                            .len(location_test_data.len())
                            .all(json!({
                                "node": expect_json::object().contains(json!({
                                    "id": expect_json::string(),
                                    "name": expect_json::string(),
                                })),
                                "cursor": expect_json::string(),
                            })),
                        "pageInfo": expect_json::object().contains(json!({
                            "hasNextPage": false,
                            "hasPrevPage": false
                        }))
                    }))
                }))
            }))
        }

        #[tokio::test]
        async fn test_relay_identifier_uuid() {
            let app = build_app();
            let server = TestServer::new(app).unwrap();
            let response = server
                .post("/graphql")
                .json(&GraphQLPayload {
                    query: ALL_CHARACTERS_QUERY.to_string(),
                    variables: Some(serde_json::Value::Object(serde_json::Map::new()))
                })
                .await;

            let character_data = get_character_test_data();

            response.assert_json(&json!({
                "data": expect_json::object().contains(json!({
                    "characters": expect_json::object().contains(json!({
                        "edges": expect_json::array().contains(
                            character_data.iter().map(|character| {
                                let identifier = RelayIdentifier::new(character.id, EntityType::Character);
                                json!({
                                    "node": expect_json::object().contains(json!({
                                        "id": identifier.to_encoded_string(),
                                        "name": character.name,
                                    })),
                                    "cursor": expect_json::string(),
                                })
                            })
                        )
                    }))
                }))
            }));
        }

        #[tokio::test]
        async fn test_relay_identifier_string() {
            let app = build_app();
            let server = TestServer::new(app).unwrap();
            let response = server
                .post("/graphql")
                .json(&GraphQLPayload {
                    query: ALL_LOCATIONS_QUERY.to_string(),
                    variables: Some(serde_json::Value::Object(serde_json::Map::new()))
                })
                .await;

            let location_data = get_location_test_data();

            response.assert_json(&json!({
                "data": expect_json::object().contains(json!({
                    "locations": expect_json::object().contains(json!({
                        "edges": expect_json::array().contains(
                            location_data.iter().map(|location| {
                                let identifier = RelayIdentifier::new(location.id.to_string(), EntityType::Location);
                                json!({
                                    "node": expect_json::object().contains(json!({
                                        "id": identifier.to_encoded_string(),
                                        "name": location.name,
                                    })),
                                    "cursor": expect_json::string(),
                                })
                            })
                        )
                    }))
                }))
            }));
        }
    }
}
