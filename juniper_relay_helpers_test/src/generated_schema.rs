#[cfg(test)]
mod integration_tests {
    use googletest::prelude::*;
    use juniper::{EmptyMutation, EmptySubscription, FieldResult, GraphQLObject, RootNode};
    use juniper_relay_helpers::{PageInfo, RelayConnection};

    // ---- Define the types ----

    #[derive(Debug, GraphQLObject, Clone, Eq, PartialEq, RelayConnection)]
    pub struct User {
        name: String,
    }

    #[derive(Debug, GraphQLObject, Clone, Eq, PartialEq, RelayConnection)]
    pub struct Post {
        title: String,
    }

    // ----- Build the query root ----

    struct QueryRoot;

    #[juniper::graphql_object()]
    impl QueryRoot {
        fn get_users() -> FieldResult<UserRelayConnection> {
            Ok(UserRelayConnection {
                count: 12,
                edges: vec![
                    UserRelayEdge {
                        node: User {
                            name: "Lune".to_owned()
                        },
                        cursor: None
                    },
                    UserRelayEdge {
                        node: User {
                            name: "Sciel".to_owned()
                        },
                        cursor: Some("some-string".to_owned())
                    }
                ],
                page_info: PageInfo {
                    start_cursor: None,
                    end_cursor: None,
                    has_prev_page: false,
                    has_next_page: false
                }
            })
        }

        fn get_posts() -> FieldResult<PostRelayConnection> {
            Ok(PostRelayConnection {
                count: 0,
                edges: vec![],
                page_info: PageInfo {
                    start_cursor: None,
                    end_cursor: None,
                    has_prev_page: false,
                    has_next_page: false
                }
            })
        }
    }

    // ---- Build the schema ----

    type Schema = RootNode<QueryRoot, EmptyMutation, EmptySubscription>;
    fn build_schema() -> Schema {
        Schema::new(QueryRoot, EmptyMutation::new(), EmptySubscription::new())
    }

    #[test]
    fn print_schema_for_debugging() {
        let schema_document = build_schema();
        let schema_sdl = schema_document.as_sdl();
        println!("{}", schema_sdl);
    }

    #[test]
    fn connection_info_generated() {
        let schema_document = build_schema();
        let schema_sdl = schema_document.as_sdl();

        assert_that!(schema_sdl, contains_substring("type UserConnection"));

        assert_that!(schema_sdl, contains_substring("type UserConnection"));
        assert_that!(schema_sdl, contains_substring("Connection type for User."));

        assert_that!(schema_sdl, contains_substring("type PostConnection"));
        assert_that!(schema_sdl, contains_substring("Connection type for Post."));
    }

    #[test]
    fn edge_info_generated() {
        let schema_document = build_schema();
        let schema_sdl = schema_document.as_sdl();

        assert_that!(schema_sdl, contains_substring("type UserEdge"));
        assert_that!(schema_sdl, contains_substring("Edge type for User."));

        assert_that!(schema_sdl, contains_substring("type PostEdge"));
        assert_that!(schema_sdl, contains_substring("Edge type for Post."));
    }

    #[test]
    fn pagination_info_generated() {
        let schema_document = build_schema();
        let schema_sdl = schema_document.as_sdl();

        assert_that!(schema_sdl, contains_substring("type PageInfo"));
        assert_that!(schema_sdl, contains_substring("Pagination information"));
    }
}
