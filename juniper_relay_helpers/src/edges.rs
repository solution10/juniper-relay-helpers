use crate::Cursor;

/// Trait encapsulating common parts of a Relay Edge.
pub trait RelayEdge {
    type NodeType;

    /// New type taking a Cursor implementation
    fn new(node: Self::NodeType, cursor: impl Cursor) -> Self;

    /// New type taking a string cursor
    fn new_raw_cursor(node: Self::NodeType, cursor: Option<String>) -> Self;
}
