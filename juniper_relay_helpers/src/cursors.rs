mod cursor;
mod offset_cursor;
mod string_cursor;

pub use cursor::*;
pub use offset_cursor::*;
pub use string_cursor::*;

#[cfg(test)]
mod tests {



    mod string_cursor_tests {
        use crate::{Cursor, StringCursor};

        #[test]
        fn test_string_cursor_raw_string() {
            let cursor = StringCursor {
                value: "some-cursor".to_string(),
            };
            assert_eq!(cursor.to_string(), "string||some-cursor");
        }

        #[test]
        fn test_string_cursor_encoded_string() {
            let cursor = StringCursor {
                value: "some-cursor".to_string(),
            };
            assert_eq!(cursor.to_encoded_string(), "c3RyaW5nfHxzb21lLWN1cnNvcg==");
        }

        #[test]
        fn test_string_cursor_from_encoded_string() {
            let cursor = StringCursor::from_encoded_string("c3RyaW5nfHxzb21lLWN1cnNvcg==").unwrap();
            assert_eq!(cursor.value, "some-cursor");
        }
    }
}
