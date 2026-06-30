use crate::Cursor;

/// Trait encapsulating common parts of a Relay Edge.
pub trait RelayEdge {
    type NodeType;
    type CursorType: Cursor;

    /// New type taking a Cursor implementation
    fn new(node: Option<Self::NodeType>, cursor: Self::CursorType) -> Self;
}
