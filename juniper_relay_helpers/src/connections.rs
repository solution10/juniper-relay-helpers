use crate::cursor_provider::CursorProvider;
use crate::{Cursor, RelayEdge};

/// Common trait for Relay connections. Will be implemented by the codegen.
pub trait RelayConnection {
    /// The type of the Edge - this will be added for you in the codegen.
    type EdgeType: RelayEdge;

    /// The underlying type of Node we're Connection-ing. Will be filled in for you by the codegen.
    type NodeType;

    /// The type of Cursor that this connection uses.
    type CursorType: Cursor;

    /// Builds a connection and associated edges from a Vec of the Nodes themselves. Pagination cursors
    /// can also be generated for you by providing the page info and CursorProvider trait instance.
    fn new<ProviderT>(
        nodes: Option<&[Option<Self::NodeType>]>,
        total_items: Option<i32>,
        cursor_provider: ProviderT,
        page_request: Option<crate::PageRequest<Self::CursorType>>,
    ) -> Self
    where
        ProviderT: CursorProvider<Self::NodeType, CursorType = Self::CursorType>;
}

#[cfg(test)]
mod tests {
    use crate::OffsetCursor;
    use juniper::GraphQLObject;
    use juniper_relay_helpers_codegen::RelayConnection;

    #[derive(Debug, GraphQLObject, RelayConnection, Clone, Eq, PartialEq)]
    #[relay(cursor = OffsetCursor)]
    pub struct User {
        name: String,
    }

    #[test]
    fn connection_types_are_generated() {
        let conn = UserRelayConnection {
            count: Some(12),
            edges: Some(vec![]),
            page_info: UserRelayConnectionPageInfo {
                start_cursor: None,
                end_cursor: None,
                has_previous_page: false,
                has_next_page: false,
            },
        };

        assert_eq!(conn.count, Some(12));
        assert_eq!(conn.edges.unwrap().len(), 0);
    }

    #[test]
    fn edge_types_are_generated() {
        let edge = UserRelayEdge {
            node: Some(User {
                name: "Lune".to_owned(),
            }),
            cursor: Some(OffsetCursor::new(527)),
        };
        assert_eq!(edge.node.unwrap().name, "Lune");
        assert_eq!(edge.cursor, Some(OffsetCursor::new(527)));
    }

    #[test]
    fn edge_implementation_new() {
        let edge = UserRelayEdge::new(
            Some(User {
                name: "Lune".to_owned(),
            }),
            OffsetCursor::new(27),
        );
        assert_eq!(edge.node.unwrap().name, "Lune");
        assert_eq!(edge.cursor, Some(OffsetCursor::new(27)));
    }
}
