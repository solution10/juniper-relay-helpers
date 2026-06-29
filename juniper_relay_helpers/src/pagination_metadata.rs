use crate::{Cursor, PageRequest};

/// Struct that holds metadata about the response that can be used in the CursorProvider
#[derive(Debug, Clone)]
pub struct PaginationMetadata<CursorType> where CursorType: Cursor {
    /// The total number of items in the result set:
    pub total_count: Option<i32>,

    /// The current PageInfo, if any:
    pub page_request: Option<PageRequest<CursorType>>,
}
