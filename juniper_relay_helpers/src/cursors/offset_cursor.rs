use crate::{CURSOR_SEGMENT_DELIMITER, Cursor, CursorError};
use juniper::GraphQLScalar;
use std::fmt::{Display, Formatter};

/// A simple offset-based cursor.
#[derive(Debug, GraphQLScalar, Default, Clone, Eq, PartialEq)]
#[graphql(
    name = "OffsetCursor",
    to_output_with = Self::to_output,
    from_input_with = Self::from_input
)]
pub struct OffsetCursor {
    /// The offset of the cursor (how many items to skip).
    pub offset: i32,
}

impl OffsetCursor {
    pub fn new(offset: i32) -> Self {
        OffsetCursor { offset }
    }

    /// Returns the "next" cursor based on adding to the current one. This is obviously not guaranteed to be
    /// valid, you need to check it first and pass in the arg.
    /// /// Passing None to `first` assumes that you requested all results, and so there cannot be a next page.
    pub fn next_page(&self, first: Option<i32>, total: u64) -> Option<OffsetCursor> {
        let first = first?;
        let last_item = self.offset.checked_add(first).unwrap_or(total as i32);

        // Have to check this here because the count - 1 can trigger and underflow and Rust is beautiful
        // and smart and refuses to do it.
        let has_next_page = if total == 0 {
            false
        } else {
            last_item < (total as i32)
        };

        if !has_next_page {
            return None;
        }

        Some(OffsetCursor::new(
            self.offset.checked_add(first).unwrap_or(0),
        ))
    }

    /// Returns the "previous" cursor based on subtracting from the current one. This is obviously not guaranteed to be
    /// valid, you need to check it first.
    /// Passing None to `first` assumes that you requested all results, and so there cannot be a prev page.
    pub fn previous_page(&self, first: Option<i32>) -> Option<OffsetCursor> {
        let first = first?;
        if self.offset > 0 {
            Some(OffsetCursor::new(
                // Safety: don't let these overflow since they're user provided.
                self.offset.checked_sub(first).unwrap_or(0).max(0), // ensure non-negative
            ))
        } else {
            None
        }
    }
}

impl Cursor for OffsetCursor {
    type CursorType = OffsetCursor;

    fn to_raw_string(&self) -> String {
        format!("offset{}{}", CURSOR_SEGMENT_DELIMITER, self.offset)
    }

    fn new(_raw: &str, parts: Vec<&str>) -> Result<OffsetCursor, CursorError> {
        if parts.len() != 2 {
            return Err(CursorError::InvalidCursor);
        }
        let offset = parts[1].parse::<i32>().unwrap_or(0);
        Ok(OffsetCursor { offset })
    }
}

impl Display for OffsetCursor {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_raw_string())
    }
}

#[cfg(test)]
mod tests {
    use crate::cursors::{Cursor, OffsetCursor};

    #[test]
    fn test_new_offset_first() {
        let cursor = OffsetCursor::new(1);
        assert_eq!(cursor.offset, 1);
    }

    #[test]
    fn test_default() {
        let cursor = OffsetCursor::default();
        assert_eq!(cursor.offset, 0);
    }

    #[test]
    fn test_raw_string() {
        let cursor = OffsetCursor { offset: 1 };
        assert_eq!(cursor.to_string(), "offset||1");
    }

    #[test]
    fn test_encoded_string() {
        let cursor = OffsetCursor { offset: 1 };
        assert_eq!(cursor.to_encoded_string(), "b2Zmc2V0fHwx");
    }

    #[test]
    fn test_from_encoded_string() {
        let cursor = OffsetCursor::from_encoded_string("b2Zmc2V0fHwx").unwrap();
        assert_eq!(cursor.offset, 1);
    }

    #[test]
    fn test_next_page_some_first() {
        let cursor = OffsetCursor::new(10);
        let next_page = cursor.next_page(Some(10), 100);

        assert!(next_page.is_some());
        let next_page = next_page.unwrap();
        assert_eq!(next_page.offset, 20);
    }

    #[test]
    fn test_next_page_none_first() {
        let cursor = OffsetCursor::new(10);
        let next_page = cursor.next_page(None, 100);
        assert!(next_page.is_none());
    }

    #[test]
    fn test_previous_page_some_first_positive() {
        let cursor = OffsetCursor::new(10);
        let prev_page = cursor.previous_page(Some(5));

        assert!(prev_page.is_some());
        let prev_page = prev_page.unwrap();
        assert_eq!(prev_page.offset, 5);
    }

    #[test]
    fn test_previous_page_some_first_below_zero() {
        let cursor = OffsetCursor::new(10);
        let prev_page = cursor.previous_page(Some(25));

        assert!(prev_page.is_some());
        let prev_page = prev_page.unwrap();
        assert_eq!(prev_page.offset, 0);
    }

    #[test]
    fn test_previous_page_none_first() {
        let cursor = OffsetCursor::new(10);
        let prev_page = cursor.previous_page(None);
        assert!(prev_page.is_none());
    }
}
