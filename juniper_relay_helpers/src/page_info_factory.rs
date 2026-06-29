use crate::Cursor;

/// Trait used by the CursorProvider's to be able to build the generated PageInfo structs from the codegen.
///
/// You shouldn't need to implement it yourself, it's just used in the generated code.
pub trait PageInfoFactory<CursorT> where CursorT: Cursor {
    fn new(has_prev_page: bool, has_next_page: bool, start_cursor: Option<CursorT>, end_cursor: Option<CursorT>) -> Self;
}