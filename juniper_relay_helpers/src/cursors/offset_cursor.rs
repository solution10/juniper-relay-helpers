use std::fmt::{Display, Formatter};
use juniper::GraphQLScalar;
use crate::{Cursor, CursorError, CURSOR_SEGMENT_DELIMITER};

/// A simple offset-based cursor.
#[derive(Debug, GraphQLScalar, Default, Clone)]
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
    fn test_offset_cursor_default() {
        let cursor = OffsetCursor::default();
        assert_eq!(cursor.offset, 0);
    }

    #[test]
    fn test_offset_cursor_raw_string() {
        let cursor = OffsetCursor {
            offset: 1,
        };
        assert_eq!(cursor.to_string(), "offset||1");
    }

    #[test]
    fn test_offset_cursor_encoded_string() {
        let cursor = OffsetCursor {
            offset: 1,
        };
        assert_eq!(cursor.to_encoded_string(), "b2Zmc2V0fHwx");
    }

    #[test]
    fn test_offset_cursor_from_encoded_string() {
        let cursor = OffsetCursor::from_encoded_string("b2Zmc2V0fHwx").unwrap();
        assert_eq!(cursor.offset, 1);
    }
}
