use juniper_relay_helpers::{Cursor, OffsetCursor, PageInfo, PageRequest};

/// Struct that holds metadata about the response that can be used in the CursorProvider
#[derive(Debug, Clone)]
pub struct PaginationMetadata {
    /// The total number of items in the result set:
    pub total_count: i32,

    /// The current PageInfo, if any:
    pub page_request: Option<PageRequest>,
}

/// Trait to implement when building a Relay cursor provider.
///
/// Cursor providers are how we generate cursors for each of the individual items
/// within the result set, without needing to do a pass and build them manually.
///
pub trait CursorProvider {
    /// Build a cursor instance for the given item, with helper metadata etc.
    ///
    /// `metadata` is information about the current resultset we're building for.
    /// `item_idx` is the index of the item we're building a cursor for.
    /// `item` is the item itself.
    fn get_cursor_for_item<T>(
        &self,
        metadata: &PaginationMetadata,
        item_idx: i32,
        item: &T
    ) -> impl Cursor;

    /// Builds the `PageInfo` to return to the RelayConnection
    fn get_page_info<T>(&self, metadata: &PaginationMetadata, items: &Vec<T>) -> PageInfo;
}


// -------------- OffsetCursorProvider ---------------

/// Built-in cursor provider that can handle Offset cursors. Serves as a reference implementation for
/// your own cursor providers too.
pub struct OffsetCursorProvider;
impl CursorProvider for OffsetCursorProvider {
    fn get_cursor_for_item<T>(&self, metadata: &PaginationMetadata, item_idx: i32, _item: &T) -> impl Cursor {
        // OK this is annoying. If there _was_ a cursor passed to `after`, the offset needs to start
        // at the next item. If there wasn't, the offset needs to start at the first item (0).
        let mut offset_adjust = 0;

        let default_cursor = OffsetCursor::default();
        let current_cursor = match &metadata.page_request {
            Some(pr) => match pr.parsed_cursor() {
                Ok(c) => match c {
                    Some(cc) => {
                        offset_adjust = 1;
                        cc
                    },
                    None => default_cursor
                },
                Err(_) => default_cursor
            },
            None => default_cursor
        };

        OffsetCursor {
            offset: current_cursor.offset + offset_adjust + item_idx,
            first: current_cursor.first
        }
    }

    fn get_page_info<T>(&self, metadata: &PaginationMetadata, items: &Vec<T>) -> PageInfo {
        let default_cursor = OffsetCursor::default();
        let current_cursor = match &metadata.page_request {
            Some(pr) => match pr.parsed_cursor() {
                Ok(c) => c.unwrap_or(default_cursor),
                Err(_) => default_cursor
            },
            None => default_cursor
        };

        let has_next_page = if let Some(pr) = &metadata.page_request {
            // Check if we requested up to or over the total items.
            if let Some(first) = pr.first {
                current_cursor.offset + first < metadata.total_count
            } else {
                false
            }
        } else {
            // We didn't request a first, which means entire result set, therefore no next page
            false
        };

        let last_index = items.len() - 1;

        PageInfo {
            has_prev_page: current_cursor.offset > 0,
            has_next_page,
            start_cursor: if items.len() > 0 {
                Some(
                    self.get_cursor_for_item(metadata, 0, &items[0])
                        .to_encoded_string()
                )
            } else {
                None
            },
            end_cursor: if items.len() > 0 {
                Some(
                    self.get_cursor_for_item(metadata, last_index as i32, &items[last_index])
                        .to_encoded_string()
                )
            } else {
                None
            },
        }
    }
}

impl OffsetCursorProvider {
    pub fn new() -> Self {
        OffsetCursorProvider
    }
}


#[cfg(test)]
mod tests {
    mod offset_cursor_provider {
        use crate::{OffsetCursorProvider, PaginationMetadata, CursorProvider, Cursor, OffsetCursor, PageRequest};

        #[derive(Debug, Clone)]
        struct Location {
            name: String
        }

        fn data() -> Vec<Location> {
            vec![
                Location { name: "Lumiére".to_owned() },
                Location { name: "Flying Waters".to_owned() }
            ]
        }

        /// Mimics a "complete" request - no `first` and no `after` with the total result set returned
        /// as part of the payload.
        #[test]
        fn test_page_info_no_request() {
            let p = OffsetCursorProvider::new();
            let pi = p.get_page_info(&PaginationMetadata {
                total_count: 2,
                page_request: None
            }, &data());

            assert_eq!(pi.has_prev_page, false);
            assert_eq!(pi.has_next_page, false);
            assert_eq!(pi.start_cursor, Some(OffsetCursor { offset: 0, first: None }.to_encoded_string()));
            assert_eq!(pi.end_cursor, Some(OffsetCursor { offset: 1, first: None }.to_encoded_string()));
        }

        /// Verifies what happens when there's a mismatch between the total count and the number of items
        /// returned from the query. Due to the lack of PageRequest, should say no next page as all results
        /// will have been returned.
        #[test]
        fn test_page_info_no_request_mismatch_results_count() {
            let p = OffsetCursorProvider::new();
            let pi = p.get_page_info(&PaginationMetadata {
                total_count: 27,
                page_request: None
            }, &data());

            assert_eq!(pi.has_prev_page, false);
            assert_eq!(pi.has_next_page, false);
            assert_eq!(pi.start_cursor, Some(OffsetCursor { offset: 0, first: None }.to_encoded_string()));
            assert_eq!(pi.end_cursor, Some(OffsetCursor { offset: 1, first: None }.to_encoded_string()));
        }

        /// Mimics a first page request - there's no `after` but there is a provided `first`
        #[test]
        fn test_page_info_has_request_first_page() {
            let p = OffsetCursorProvider::new();
            let pi = p.get_page_info(&PaginationMetadata {
                total_count: 27,
                page_request: Some(
                    PageRequest {
                        first: Some(10),
                        after: None
                    }
                )
            }, &data());

            assert_eq!(pi.has_prev_page, false);
            assert_eq!(pi.has_next_page, true);
            assert_eq!(pi.start_cursor, Some(OffsetCursor { offset: 0, first: None }.to_encoded_string()));
            assert_eq!(pi.end_cursor, Some(OffsetCursor { offset: 1, first: None }.to_encoded_string()));
        }

        /// Test mimics pagination through a full set of results
        #[test]
        fn test_page_info_paginating_through_set() {
            let p = OffsetCursorProvider::new();
            let total_items = 13;
            let data = vec![
                Location { name: "Lumiére".to_owned() },
                Location { name: "Spring Meadows".to_owned() },
                Location { name: "Flying Waters".to_owned() },
                Location { name: "Gestral Village".to_owned() },
                Location { name: "Stone Wave Cliffs".to_owned() }
            ];

            let pi1 = p.get_page_info(&PaginationMetadata {
                total_count: total_items,
                page_request: Some(
                    PageRequest {
                        first: Some(5),
                        after: None
                    }
                )
            }, &data);
            assert_eq!(pi1.has_prev_page, false);
            assert_eq!(pi1.has_next_page, true);
            assert_eq!(pi1.start_cursor, Some(OffsetCursor { offset: 0, first: None }.to_encoded_string()));
            assert_eq!(pi1.end_cursor, Some(OffsetCursor { offset: 4, first: None }.to_encoded_string()));

            let pi2 = p.get_page_info(&PaginationMetadata {
                total_count: total_items,
                page_request: Some(
                    PageRequest {
                        first: Some(5),
                        after: pi1.end_cursor.clone()
                    }
                )
            }, &data);
            assert_eq!(pi2.has_prev_page, true);
            assert_eq!(pi2.has_next_page, true);
            assert_eq!(pi2.start_cursor, Some(OffsetCursor { offset: 5, first: None }.to_encoded_string()));
            assert_eq!(pi2.end_cursor, Some(OffsetCursor { offset: 9, first: None }.to_encoded_string()));

            let pi3 = p.get_page_info(&PaginationMetadata {
                total_count: total_items,
                page_request: Some(
                    PageRequest {
                        first: Some(5),
                        after: pi2.end_cursor.clone()
                    }
                )
            }, &vec![data[0].clone(), data[1].clone(), data[2].clone()]);
            assert_eq!(pi3.has_prev_page, true);
            assert_eq!(pi3.has_next_page, false);
            assert_eq!(pi3.start_cursor, Some(OffsetCursor { offset: 10, first: None }.to_encoded_string()));
            assert_eq!(pi3.end_cursor, Some(OffsetCursor { offset: 12, first: None }.to_encoded_string()));
        }

       //  /// Mimics a subsequent page request - there's an `after` and a `first`
         // /// TODO: I think this is actually an off-by-one error :yikes:
        // #[test]
        // fn test_page_info_has_request_subsequent_page() {
        //     let p = OffsetCursorProvider::new();
        //     let pi = p.get_page_info(&PaginationMetadata {
        //         total_count: 27,
        //         page_request: Some(
        //             PageRequest {
        //                 first: Some(10),
        //                 after: Some(OffsetCursor { offset: 9, first: Some(10) }.to_encoded_string())
        //             }
        //         )
        //     }, &data());
        //
        //     assert_eq!(pi.has_prev_page, true);
        //     assert_eq!(pi.has_next_page, true);
        //     assert_eq!(pi.start_cursor, Some(OffsetCursor { offset: 9, first: Some(10) }.to_encoded_string()));
        //     assert_eq!(pi.end_cursor, Some(OffsetCursor { offset: 10, first: Some(10) }.to_encoded_string()));
        // }
    }
}

