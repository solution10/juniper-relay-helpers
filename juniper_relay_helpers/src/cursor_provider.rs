use crate::StringCursor;
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
pub trait CursorProvider<ItemT> {
    /// Build a cursor instance for the given item, with helper metadata etc.
    ///
    /// `metadata` is information about the current resultset we're building for.
    /// `item_idx` is the index of the item we're building a cursor for.
    /// `item` is the item itself.
    fn get_cursor_for_item(
        &self,
        metadata: &PaginationMetadata,
        item_idx: i32,
        item: &ItemT,
    ) -> impl Cursor;

    /// Builds the `PageInfo` to return to the RelayConnection
    fn get_page_info(&self, metadata: &PaginationMetadata, items: &[ItemT]) -> PageInfo;
}

// -------------- OffsetCursorProvider ---------------

/// Built-in cursor provider that can handle Offset cursors. Serves as a reference implementation for
/// your own cursor providers too.
pub struct OffsetCursorProvider;
impl<ItemT> CursorProvider<ItemT> for OffsetCursorProvider {
    fn get_cursor_for_item(
        &self,
        metadata: &PaginationMetadata,
        item_idx: i32,
        _item: &ItemT,
    ) -> impl Cursor {
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
                    }
                    None => default_cursor,
                },
                Err(_) => default_cursor,
            },
            None => default_cursor,
        };

        OffsetCursor {
            offset: current_cursor.offset + offset_adjust + item_idx,
            first: current_cursor.first,
        }
    }

    fn get_page_info(&self, metadata: &PaginationMetadata, items: &[ItemT]) -> PageInfo {
        let default_cursor = OffsetCursor::default();
        let current_cursor = match &metadata.page_request {
            Some(pr) => match pr.parsed_cursor() {
                Ok(c) => c.unwrap_or(default_cursor),
                Err(_) => default_cursor,
            },
            None => default_cursor,
        };

        let has_next_page = if let Some(pr) = &metadata.page_request {
            // Check if we requested up to or over the total items.
            if let Some(first) = pr.first {
                current_cursor.offset + first < metadata.total_count
            } else {
                false
            }
        } else {
            // We didn't request a first, which means the entire result set, therefore no next page
            false
        };

        let last_index = items.len() - 1;

        PageInfo {
            has_prev_page: current_cursor.offset > 0,
            has_next_page,
            start_cursor: if !items.is_empty() {
                Some(
                    self.get_cursor_for_item(metadata, 0, &items[0])
                        .to_encoded_string(),
                )
            } else {
                None
            },
            end_cursor: if !items.is_empty() {
                Some(
                    self.get_cursor_for_item(metadata, last_index as i32, &items[last_index])
                        .to_encoded_string(),
                )
            } else {
                None
            },
        }
    }
}

impl Default for OffsetCursorProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl OffsetCursorProvider {
    /// Shortcut method for creating a new instance of OffsetCursorProvider. May be used in the future
    /// if we need to pass things into it.
    pub fn new() -> Self {
        OffsetCursorProvider
    }
}

// ------------- Keyed cursor provider -------------

/// Trait to implement to use with items in the `KeyedCursorProvider`.
trait CursorByKey {
    fn cursor_key(&self) -> String;
}

/// This cursor provider is for working with something like DynamoDB. Each item's cursor is generated
/// by implementing the `CursorByKey` trait, and the PageInfo is generated using the item cursors themselves.
///
/// If any `after` is provided, it's assumed that there is a previous page.
/// If there are any items returned, it's assumed that there is a next page.
///
/// NOTE - read that previous line again. This follows the style of opaque, web scale cursors where the only
/// valid last page is an empty page. This can be unexpected to a lot of frontends.
pub struct KeyedCursorProvider;

impl<ItemT> CursorProvider<ItemT> for KeyedCursorProvider
where
    ItemT: CursorByKey,
{
    fn get_cursor_for_item(
        &self,
        _metadata: &PaginationMetadata,
        _item_idx: i32,
        item: &ItemT,
    ) -> impl Cursor {
        StringCursor::new(item.cursor_key())
    }

    fn get_page_info(&self, metadata: &PaginationMetadata, items: &[ItemT]) -> PageInfo {
        let mut first_item_cursor: Option<String> = None;
        let mut last_item_cursor: Option<String> = None;

        if let Some(first_item) = items.first() {
            first_item_cursor = Some(
                self.get_cursor_for_item(metadata, 0, first_item)
                    .to_encoded_string(),
            );
        }

        if let Some(last_item) = items.last() {
            last_item_cursor = Some(
                self.get_cursor_for_item(metadata, items.len() as i32 - 1, last_item)
                    .to_encoded_string(),
            );
        }

        let mut has_previous_page = false;
        if let Some(pr) = &metadata.page_request
            && pr.after.is_some() {
                has_previous_page = true;
            }

        PageInfo {
            start_cursor: first_item_cursor,
            end_cursor: last_item_cursor,
            has_prev_page: has_previous_page,
            has_next_page: !items.is_empty(),
        }
    }
}

#[cfg(test)]
mod tests {
    mod offset_cursor_provider {
        use crate::{
            Cursor, CursorProvider, OffsetCursor, OffsetCursorProvider, PageRequest,
            PaginationMetadata,
        };

        #[derive(Debug, Clone)]
        struct Location {
            #[allow(dead_code)]
            name: String,
        }

        fn data() -> Vec<Location> {
            vec![
                Location {
                    name: "Lumiére".to_owned(),
                },
                Location {
                    name: "Flying Waters".to_owned(),
                },
            ]
        }

        /// Mimics a "complete" request - no `first` and no `after` with the total result set returned
        /// as part of the payload.
        #[test]
        fn test_page_info_no_request() {
            let p = OffsetCursorProvider::new();
            let pi = p.get_page_info(
                &PaginationMetadata {
                    total_count: 2,
                    page_request: None,
                },
                &data(),
            );

            assert!(!pi.has_prev_page);
            assert!(!pi.has_next_page);
            assert_eq!(
                pi.start_cursor,
                Some(
                    OffsetCursor {
                        offset: 0,
                        first: None
                    }
                    .to_encoded_string()
                )
            );
            assert_eq!(
                pi.end_cursor,
                Some(
                    OffsetCursor {
                        offset: 1,
                        first: None
                    }
                    .to_encoded_string()
                )
            );
        }

        /// Verifies what happens when there's a mismatch between the total count and the number of items
        /// returned from the query. Due to the lack of PageRequest, should say no next page as all results
        /// will have been returned.
        #[test]
        fn test_page_info_no_request_mismatch_results_count() {
            let p = OffsetCursorProvider::new();
            let pi = p.get_page_info(
                &PaginationMetadata {
                    total_count: 27,
                    page_request: None,
                },
                &data(),
            );

            assert!(!pi.has_prev_page);
            assert!(!pi.has_next_page);
            assert_eq!(
                pi.start_cursor,
                Some(
                    OffsetCursor {
                        offset: 0,
                        first: None
                    }
                    .to_encoded_string()
                )
            );
            assert_eq!(
                pi.end_cursor,
                Some(
                    OffsetCursor {
                        offset: 1,
                        first: None
                    }
                    .to_encoded_string()
                )
            );
        }

        /// Mimics a first page request - there's no `after` but there is a provided `first`
        #[test]
        fn test_page_info_has_request_first_page() {
            let p = OffsetCursorProvider::new();
            let pi = p.get_page_info(
                &PaginationMetadata {
                    total_count: 27,
                    page_request: Some(PageRequest {
                        first: Some(10),
                        after: None,
                    }),
                },
                &data(),
            );

            assert!(!pi.has_prev_page);
            assert!(pi.has_next_page);
            assert_eq!(
                pi.start_cursor,
                Some(
                    OffsetCursor {
                        offset: 0,
                        first: None
                    }
                    .to_encoded_string()
                )
            );
            assert_eq!(
                pi.end_cursor,
                Some(
                    OffsetCursor {
                        offset: 1,
                        first: None
                    }
                    .to_encoded_string()
                )
            );
        }

        /// Test mimics pagination through a full set of results
        #[test]
        fn test_page_info_paginating_through_set() {
            let p = OffsetCursorProvider::new();
            let total_items = 13;
            let data = vec![
                Location {
                    name: "Lumiére".to_owned(),
                },
                Location {
                    name: "Spring Meadows".to_owned(),
                },
                Location {
                    name: "Flying Waters".to_owned(),
                },
                Location {
                    name: "Gestral Village".to_owned(),
                },
                Location {
                    name: "Stone Wave Cliffs".to_owned(),
                },
            ];

            let pi1 = p.get_page_info(
                &PaginationMetadata {
                    total_count: total_items,
                    page_request: Some(PageRequest {
                        first: Some(5),
                        after: None,
                    }),
                },
                &data,
            );
            assert!(!pi1.has_prev_page);
            assert!(pi1.has_next_page);
            assert_eq!(
                pi1.start_cursor,
                Some(
                    OffsetCursor {
                        offset: 0,
                        first: None
                    }
                    .to_encoded_string()
                )
            );
            assert_eq!(
                pi1.end_cursor,
                Some(
                    OffsetCursor {
                        offset: 4,
                        first: None
                    }
                    .to_encoded_string()
                )
            );

            let pi2 = p.get_page_info(
                &PaginationMetadata {
                    total_count: total_items,
                    page_request: Some(PageRequest {
                        first: Some(5),
                        after: pi1.end_cursor.clone(),
                    }),
                },
                &data,
            );
            assert!(pi2.has_prev_page);
            assert!(pi2.has_next_page);
            assert_eq!(
                pi2.start_cursor,
                Some(
                    OffsetCursor {
                        offset: 5,
                        first: None
                    }
                    .to_encoded_string()
                )
            );
            assert_eq!(
                pi2.end_cursor,
                Some(
                    OffsetCursor {
                        offset: 9,
                        first: None
                    }
                    .to_encoded_string()
                )
            );

            let pi3 = p.get_page_info(
                &PaginationMetadata {
                    total_count: total_items,
                    page_request: Some(PageRequest {
                        first: Some(5),
                        after: pi2.end_cursor.clone(),
                    }),
                },
                &[data[0].clone(), data[1].clone(), data[2].clone()],
            );
            assert!(pi3.has_prev_page);
            assert!(!pi3.has_next_page);
            assert_eq!(
                pi3.start_cursor,
                Some(
                    OffsetCursor {
                        offset: 10,
                        first: None
                    }
                    .to_encoded_string()
                )
            );
            assert_eq!(
                pi3.end_cursor,
                Some(
                    OffsetCursor {
                        offset: 12,
                        first: None
                    }
                    .to_encoded_string()
                )
            );
        }
    }

    mod keyed_cursor_provider {
        use crate::{
            Cursor, CursorProvider, KeyedCursorProvider, PageRequest, PaginationMetadata,
            RelayConnection, StringCursor,
        };
        use juniper::GraphQLObject;
        use juniper_relay_helpers::cursor_provider::CursorByKey;

        #[derive(Debug, Clone, GraphQLObject, RelayConnection, Eq, PartialEq)]
        pub struct NoSQLItem {
            id: String,
        }
        impl CursorByKey for NoSQLItem {
            fn cursor_key(&self) -> String {
                self.id.to_string()
            }
        }

        #[test]
        fn test_item_cursors() {
            let p = KeyedCursorProvider;
            let items = [NoSQLItem {
                    id: "id-1".to_string(),
                },
                NoSQLItem {
                    id: "id-2".to_string(),
                },
                NoSQLItem {
                    id: "id-3".to_string(),
                }];

            let meta = PaginationMetadata {
                total_count: 3,
                page_request: Some(PageRequest::new(
                    Some(10),
                    Some(StringCursor::new("".to_string())),
                )),
            };

            let i1_cursor = p.get_cursor_for_item(&meta, 0, &items[0]);
            assert_eq!(i1_cursor.to_encoded_string(), "c3RyaW5nOmlkLTE=");

            let i2_cursor = p.get_cursor_for_item(&meta, 1, &items[1]);
            assert_eq!(i2_cursor.to_encoded_string(), "c3RyaW5nOmlkLTI=");

            let i3_cursor = p.get_cursor_for_item(&meta, 2, &items[2]);
            assert_eq!(i3_cursor.to_encoded_string(), "c3RyaW5nOmlkLTM=");
        }

        #[test]
        fn test_page_info_full_page() {
            let p = KeyedCursorProvider {};
            let items = vec![
                NoSQLItem {
                    id: "id-1".to_string(),
                },
                NoSQLItem {
                    id: "id-2".to_string(),
                },
                NoSQLItem {
                    id: "id-3".to_string(),
                },
            ];

            let meta = PaginationMetadata {
                total_count: 3,
                page_request: Some(PageRequest {
                    first: Some(10),
                    after: None,
                }),
            };

            let page_info = p.get_page_info(&meta, &items);
            assert!(!page_info.has_prev_page);
            assert!(page_info.has_next_page); // assume next is true due to items being returned.
            assert_eq!(page_info.start_cursor, Some("c3RyaW5nOmlkLTE=".to_string()));
            assert_eq!(page_info.end_cursor, Some("c3RyaW5nOmlkLTM=".to_string()));
        }

        #[test]
        fn test_page_info_first_page_of_many() {
            let p = KeyedCursorProvider {};
            let items = vec![
                NoSQLItem {
                    id: "id-1".to_string(),
                },
                NoSQLItem {
                    id: "id-2".to_string(),
                },
                NoSQLItem {
                    id: "id-3".to_string(),
                },
            ];

            let meta = PaginationMetadata {
                total_count: 30, // More than the items returned, we have more items
                page_request: Some(PageRequest {
                    first: Some(10),
                    after: None,
                }),
            };

            let page_info = p.get_page_info(&meta, &items);
            assert!(!page_info.has_prev_page);
            assert!(page_info.has_next_page);
            assert_eq!(page_info.start_cursor, Some("c3RyaW5nOmlkLTE=".to_string()));
            assert_eq!(page_info.end_cursor, Some("c3RyaW5nOmlkLTM=".to_string()));
        }

        #[test]
        fn test_page_info_last_page() {
            let p = KeyedCursorProvider {};
            let items: Vec<NoSQLItem> = vec![];

            let meta = PaginationMetadata {
                total_count: 30, // More than the items returned, we have more items
                page_request: Some(PageRequest {
                    first: Some(10),                             // More than items returned
                    after: Some("c3RyaW5nOmlkLTA=".to_string()), // id-0 - we're paginating.
                }),
            };

            let page_info = p.get_page_info(&meta, &items);
            assert!(page_info.has_prev_page);
            assert!(!page_info.has_next_page);
            assert_eq!(page_info.start_cursor, None);
            assert_eq!(page_info.end_cursor, None);
        }
    }
}
