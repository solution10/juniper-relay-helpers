use crate::pagination_metadata::PaginationMetadata;
use crate::{PageInfoFactory, StringCursor};
use juniper_relay_helpers::{Cursor, OffsetCursor};

/// Trait to implement when building a Relay cursor provider.
///
/// Cursor providers are how we generate cursors for each of the individual items
/// within the result set, without needing to do a pass and build them manually.
///
pub trait CursorProvider<ItemT> {
    type CursorType: Cursor;

    /// Build a cursor instance for the given item, with helper metadata etc.
    ///
    /// `metadata` is information about the current resultset we're building for.
    /// `item_idx` is the index of the item we're building a cursor for.
    /// `item` is the item itself.
    fn get_cursor_for_item(
        &self,
        metadata: &PaginationMetadata<Self::CursorType>,
        item_idx: i32,
        item: &ItemT,
    ) -> Self::CursorType;

    /// Builds the `PageInfo` to return to the RelayConnection
    fn get_page_info<PageInfoType>(
        &self,
        metadata: &PaginationMetadata<Self::CursorType>,
        items: &[ItemT],
    ) -> PageInfoType
    where
        PageInfoType: PageInfoFactory<Self::CursorType>;
}

// -------------- OffsetCursorProvider ---------------

/// Built-in cursor provider that can handle Offset cursors. Serves as a reference implementation for
/// your own cursor providers too.
pub struct OffsetCursorProvider;
impl<ItemT> CursorProvider<ItemT> for OffsetCursorProvider {
    type CursorType = OffsetCursor;

    fn get_cursor_for_item(
        &self,
        metadata: &PaginationMetadata<OffsetCursor>,
        item_idx: i32,
        _item: &ItemT,
    ) -> OffsetCursor {
        // OK this is annoying. If there _was_ a cursor passed to `after`, the offset needs to start
        // at the next item. If there wasn't, the offset needs to start at the first item (0).
        let mut offset_adjust = 0;

        let default_cursor = OffsetCursor::default();
        let current_cursor = match &metadata.page_request {
            Some(pr) => match pr.current_cursor() {
                Some(cc) => {
                    offset_adjust = 1;
                    cc
                }
                None => default_cursor,
            },
            None => default_cursor,
        };

        OffsetCursor::new(current_cursor.offset + offset_adjust + item_idx)
    }

    fn get_page_info<PageInfoType>(
        &self,
        metadata: &PaginationMetadata<OffsetCursor>,
        items: &[ItemT],
    ) -> PageInfoType
    where
        PageInfoType: PageInfoFactory<OffsetCursor>,
    {
        let default_cursor = OffsetCursor::default();
        let current_cursor = metadata
            .clone()
            .page_request
            .and_then(|pr| pr.current_cursor())
            .unwrap_or(default_cursor);

        let has_next_page = if let Some(pr) = &metadata.page_request {
            // Check if we requested up to or over the total items.
            if let Some(first) = pr.first {
                current_cursor.offset + first < metadata.total_count.unwrap_or(0)
            } else {
                false
            }
        } else {
            // We didn't request a first, which means the entire result set, therefore no next page
            false
        };

        PageInfoType::new(
            current_cursor.offset > 0,
            has_next_page,
            if !items.is_empty() {
                Some(self.get_cursor_for_item(metadata, 0, &items[0]))
            } else {
                None
            },
            if !items.is_empty() {
                let last_index = items.len() - 1;
                Some(self.get_cursor_for_item(metadata, last_index as i32, &items[last_index]))
            } else {
                None
            },
        )
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
pub trait CursorByKey {
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
    type CursorType = StringCursor;

    fn get_cursor_for_item(
        &self,
        _metadata: &PaginationMetadata<StringCursor>,
        _item_idx: i32,
        item: &ItemT,
    ) -> StringCursor {
        StringCursor::new(item.cursor_key())
    }

    fn get_page_info<PageInfoType>(
        &self,
        metadata: &PaginationMetadata<StringCursor>,
        items: &[ItemT],
    ) -> PageInfoType
    where
        PageInfoType: PageInfoFactory<StringCursor>,
    {
        let mut first_item_cursor: Option<StringCursor> = None;
        let mut last_item_cursor: Option<StringCursor> = None;

        if let Some(first_item) = items.first() {
            first_item_cursor = Some(self.get_cursor_for_item(metadata, 0, first_item));
        }

        if let Some(last_item) = items.last() {
            last_item_cursor =
                Some(self.get_cursor_for_item(metadata, items.len() as i32 - 1, last_item));
        }

        let mut has_previous_page = false;
        if let Some(pr) = &metadata.page_request
            && pr.after.is_some()
        {
            has_previous_page = true;
        }

        PageInfoType::new(
            has_previous_page,
            !items.is_empty(),
            first_item_cursor,
            last_item_cursor,
        )
    }
}

#[cfg(test)]
mod tests {
    mod offset_cursor_provider {
        use crate::{
            CursorProvider, OffsetCursor, OffsetCursorProvider, PageRequest, PaginationMetadata,
        };
        use juniper::GraphQLObject;
        use juniper_relay_helpers_codegen::RelayConnection;

        #[derive(Debug, Clone, GraphQLObject, RelayConnection)]
        #[relay(cursor = OffsetCursor)]
        pub struct Location {
            pub name: String,
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
            let pi = p.get_page_info::<LocationRelayConnectionPageInfo>(
                &PaginationMetadata {
                    total_count: Some(2),
                    page_request: None,
                },
                &data(),
            );

            assert!(!pi.has_prev_page);
            assert!(!pi.has_next_page);
            assert_eq!(pi.start_cursor, Some(OffsetCursor::new(0)));
            assert_eq!(pi.end_cursor, Some(OffsetCursor::new(1)));
        }

        /// Verifies what happens when there's a mismatch between the total count and the number of items
        /// returned from the query. Due to the lack of PageRequest, should say no next page as all results
        /// will have been returned.
        #[test]
        fn test_page_info_no_request_mismatch_results_count() {
            let p = OffsetCursorProvider::new();
            let pi = p.get_page_info::<LocationRelayConnectionPageInfo>(
                &PaginationMetadata {
                    total_count: Some(27),
                    page_request: None,
                },
                &data(),
            );

            assert!(!pi.has_prev_page);
            assert!(!pi.has_next_page);
            assert_eq!(pi.start_cursor, Some(OffsetCursor::new(0)));
            assert_eq!(pi.end_cursor, Some(OffsetCursor::new(1)));
        }

        /// Mimics a first page request - there's no `after` but there is a provided `first`
        #[test]
        fn test_page_info_has_request_first_page() {
            let p = OffsetCursorProvider::new();
            let pi = p.get_page_info::<LocationRelayConnectionPageInfo>(
                &PaginationMetadata {
                    total_count: Some(27),
                    page_request: Some(PageRequest {
                        first: Some(10),
                        after: None,
                        before: None,
                    }),
                },
                &data(),
            );

            assert!(!pi.has_prev_page);
            assert!(pi.has_next_page);
            assert_eq!(pi.start_cursor, Some(OffsetCursor::new(0)));
            assert_eq!(pi.end_cursor, Some(OffsetCursor::new(1)));
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

            let pi1 = p.get_page_info::<LocationRelayConnectionPageInfo>(
                &PaginationMetadata {
                    total_count: Some(total_items),
                    page_request: Some(PageRequest {
                        first: Some(5),
                        after: None,
                        before: None,
                    }),
                },
                &data,
            );
            assert!(!pi1.has_prev_page);
            assert!(pi1.has_next_page);
            assert_eq!(pi1.start_cursor, Some(OffsetCursor::new(0)));
            assert_eq!(pi1.end_cursor, Some(OffsetCursor::new(4)));

            let pi2 = p.get_page_info::<LocationRelayConnectionPageInfo>(
                &PaginationMetadata {
                    total_count: Some(total_items),
                    page_request: Some(PageRequest {
                        first: Some(5),
                        after: pi1.end_cursor.clone(),
                        before: None,
                    }),
                },
                &data,
            );
            assert!(pi2.has_prev_page);
            assert!(pi2.has_next_page);
            assert_eq!(pi2.start_cursor, Some(OffsetCursor::new(5)));
            assert_eq!(pi2.end_cursor, Some(OffsetCursor::new(9)));

            let pi3 = p.get_page_info::<LocationRelayConnectionPageInfo>(
                &PaginationMetadata {
                    total_count: Some(total_items),
                    page_request: Some(PageRequest {
                        first: Some(5),
                        after: pi2.end_cursor.clone(),
                        before: None,
                    }),
                },
                &[data[0].clone(), data[1].clone(), data[2].clone()],
            );
            assert!(pi3.has_prev_page);
            assert!(!pi3.has_next_page);
            assert_eq!(pi3.start_cursor, Some(OffsetCursor::new(10)));
            assert_eq!(pi3.end_cursor, Some(OffsetCursor::new(12)));
        }

        #[test]
        fn test_page_info_empty_list() {
            let p = OffsetCursorProvider::new();
            let total_items = 0;
            let data: Vec<String> = vec![];

            let pi1 = p.get_page_info::<LocationRelayConnectionPageInfo>(
                &PaginationMetadata {
                    total_count: Some(total_items),
                    page_request: Some(PageRequest {
                        first: Some(5),
                        after: None,
                        before: None,
                    }),
                },
                &data,
            );
            assert!(!pi1.has_prev_page);
            assert!(!pi1.has_next_page);
            assert_eq!(pi1.start_cursor, None);
            assert_eq!(pi1.end_cursor, None);
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
            let items = [
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
                total_count: Some(3),
                page_request: Some(PageRequest::new(
                    Some(10),
                    Some(StringCursor::new("".to_string())),
                    None,
                )),
            };

            let i1_cursor = p.get_cursor_for_item(&meta, 0, &items[0]);
            assert_eq!(i1_cursor.to_encoded_string(), "c3RyaW5nfHxpZC0x");

            let i2_cursor = p.get_cursor_for_item(&meta, 1, &items[1]);
            assert_eq!(i2_cursor.to_encoded_string(), "c3RyaW5nfHxpZC0y");

            let i3_cursor = p.get_cursor_for_item(&meta, 2, &items[2]);
            assert_eq!(i3_cursor.to_encoded_string(), "c3RyaW5nfHxpZC0z");
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
                total_count: Some(3),
                page_request: Some(PageRequest {
                    first: Some(10),
                    after: None,
                    before: None,
                }),
            };

            let page_info = p.get_page_info::<NoSQLItemRelayConnectionPageInfo>(&meta, &items);
            assert!(!page_info.has_prev_page);
            assert!(page_info.has_next_page); // assume next is true due to items being returned.
            assert_eq!(
                page_info.start_cursor,
                Some(StringCursor::new("id-1".to_string()))
            );
            assert_eq!(
                page_info.end_cursor,
                Some(StringCursor::new("id-3".to_string()))
            );
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
                total_count: Some(30), // More than the items returned, we have more items
                page_request: Some(PageRequest {
                    first: Some(10),
                    after: None,
                    before: None,
                }),
            };

            let page_info = p.get_page_info::<NoSQLItemRelayConnectionPageInfo>(&meta, &items);
            assert!(!page_info.has_prev_page);
            assert!(page_info.has_next_page);
            assert_eq!(
                page_info.start_cursor,
                Some(StringCursor::new("id-1".to_string()))
            );
            assert_eq!(
                page_info.end_cursor,
                Some(StringCursor::new("id-3".to_string()))
            );
        }

        #[test]
        fn test_page_info_last_page() {
            let p = KeyedCursorProvider {};
            let items: Vec<NoSQLItem> = vec![];

            let meta = PaginationMetadata {
                total_count: Some(30), // More than the items returned, we have more items
                page_request: Some(PageRequest {
                    first: Some(10), // More than items returned
                    after: Some(StringCursor::new("c3RyaW5nOmlkLTA=".to_string())), // id-0 - we're paginating.
                    before: None,
                }),
            };

            let page_info = p.get_page_info::<NoSQLItemRelayConnectionPageInfo>(&meta, &items);
            assert!(page_info.has_prev_page);
            assert!(!page_info.has_next_page);
            assert_eq!(page_info.start_cursor, None);
            assert_eq!(page_info.end_cursor, None);
        }
    }
}
