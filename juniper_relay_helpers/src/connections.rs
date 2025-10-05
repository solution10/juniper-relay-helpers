use crate::cursor_provider::{CursorProvider};
use crate::{RelayEdge};

/// Common trait for Relay connections. Will be implemented by the codegen.
pub trait RelayConnection {
    /// The type of the Edge - this will be added for you in the codegen.
    type EdgeType: RelayEdge;

    /// The underlying type of Node we're Connection-ing. Will be filled in for you by the codegen.
    type NodeType;

    /// Builds a connection and associated edges from a Vec of the Nodes themselves. Pagination cursors
    /// can also be generated for you by providing the page info and CursorProvider trait instance.
    fn new(
        nodes: &Vec<Self::NodeType>,
        total_items: i32,
        cursor_provider: impl CursorProvider,
        page_request: Option<crate::PageRequest>
    ) -> Self;
}

#[cfg(test)]
mod tests {
    use juniper::GraphQLObject;
    use juniper_relay_helpers_codegen::{RelayConnection};
    use crate::{OffsetCursor, PageInfo};

    #[derive(Debug, GraphQLObject, RelayConnection, Clone, Eq, PartialEq)]
    pub struct User {
        name: String,
    }

    #[test]
    fn connection_types_are_generated() {
        let conn = UserRelayConnection {
            count: 12,
            edges: vec![],
            page_info: PageInfo {
                start_cursor: None,
                end_cursor: None,
                has_prev_page: false,
                has_next_page: false
            }
        };

        assert_eq!(conn.count, 12);
        assert_eq!(conn.edges.len(), 0);
    }

    // #[test]
    // fn connection_new_generated() {
    //     let conn = UserRelayConnection::new(
    //         vec![
    //             User { name: "Lune".to_owned() },
    //         ],
    //         127,
    //         todo!(),
    //         None
    //     );
    // }

    #[test]
    fn edge_types_are_generated() {
        let edge = UserRelayEdge {
            node: User {
                name: "Lune".to_owned(),
            },
            cursor: Some("some-string".to_owned())
        };
        assert_eq!(edge.node.name, "Lune");
        assert_eq!(edge.cursor, Some("some-string".to_owned()));
    }

    #[test]
    fn edge_implementation_new() {
        let edge = UserRelayEdge::new(User {
            name: "Lune".to_owned(),
        }, OffsetCursor { offset: 0, first: Some(10) });
        assert_eq!(edge.node.name, "Lune");
        assert_eq!(edge.cursor, Some("b2Zmc2V0OjA6MTA=".into()));

        let edge2 = UserRelayEdge::new_raw_cursor(User {
            name: "Sciel".to_owned(),
        }, Some("some-cursor".to_owned()));
        assert_eq!(edge2.node.name, "Sciel");
        assert_eq!(edge2.cursor, Some("some-cursor".into()));
    }
}
