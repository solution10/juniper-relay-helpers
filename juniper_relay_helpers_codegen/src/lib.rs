use proc_macro2::{Ident, Span};
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput};

/// Macro that will generate Connection and Edge structs for you to use when returning lists.
#[proc_macro_derive(RelayConnection)]
pub fn macro_relay_connection_node(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let out = match input.data {
        Data::Struct(_s) => {
            let connection_gql_name = format!("{}Connection", input.ident);
            let connection_gql_desc = format!("Connection type for {}.", input.ident);
            let edge_gql_name = format!("{}Edge", input.ident);
            let edge_gql_desc = format!("Edge type for {}.", input.ident);
            let connection_name = Ident::new(&format!("{}RelayConnection", input.ident), Span::mixed_site());
            let edge_name = Ident::new(&format!("{}RelayEdge", input.ident), Span::mixed_site());
            let edge_trait_name = Ident::new(&format!("{}RelayEdgeTrait", input.ident), Span::mixed_site());
            let struct_name = input.ident;

            quote! {
                #[derive(juniper::GraphQLObject, Debug, Clone, Eq, PartialEq)]
                #[graphql(
                    name = #connection_gql_name,
                    description = #connection_gql_desc
                )]
                pub struct #connection_name {
                    pub count: i32,
                    pub edges: Vec<#edge_name>,
                    pub page_info: juniper_relay_helpers::PageInfo,
                }

                use juniper_relay_helpers::RelayEdge as #edge_trait_name;
                impl juniper_relay_helpers::RelayConnection for #connection_name {
                    type EdgeType = #edge_name;
                    type NodeType = #struct_name;

                    fn new(
                        nodes: &Vec<#struct_name>,
                        total_items: i32,
                        cursor_provider: impl juniper_relay_helpers::CursorProvider,
                        page_request: Option<juniper_relay_helpers::PageRequest>
                    ) -> Self {
                        let metadata = juniper_relay_helpers::PaginationMetadata {
                            total_count: total_items,
                            page_request
                        };
                        Self {
                            count: total_items,
                            edges: nodes.iter().enumerate().map(|(idx, node)| {
                                #edge_name::new(
                                    node.clone(),
                                    cursor_provider.get_cursor_for_item(&metadata, idx as i32, node)
                                )
                            }).collect(),
                            page_info: cursor_provider.get_page_info(&metadata, &nodes),
                        }
                    }
                }

                #[derive(juniper::GraphQLObject, Debug, Clone, Eq, PartialEq)]
                #[graphql(
                    name = #edge_gql_name,
                    description = #edge_gql_desc
                )]
                pub struct #edge_name {
                    pub node: #struct_name,
                    pub cursor: Option<String>,
                }

                impl juniper_relay_helpers::RelayEdge for #edge_name {
                    type NodeType = #struct_name;
                    fn new(node: Self::NodeType, cursor: impl juniper_relay_helpers::Cursor) -> Self {
                        Self {
                            node: node,
                            cursor: Some(cursor.to_encoded_string()),
                        }
                    }

                    fn new_raw_cursor(node: Self::NodeType, cursor: Option<String>) -> Self {
                        Self {
                            node: node,
                            cursor: cursor,
                        }
                    }
                }
            }
        }
        _ => quote! {}
    };

    out.into()
}

/// Macro for extending an Enum with the traits required for it to be used as a type discriminator
/// within a relay identifier.
///
/// Equivalent to implementing `Display` and `FromStr` yourself, just saves you the hassle.
///
/// Allows:
///
/// ```nocompile
/// use crate::IdentifierTypeDiscriminator;
///
/// #[derive(IdentifierTypeDiscriminator)]
/// enum EntityType {
///     Character,
///     Weapon,
/// }
/// ```
///
/// `EntityType` can now be used in `RelayIdentifier(123, EntityType::Character)` without
/// any additional code.
///
#[proc_macro_derive(IdentifierTypeDiscriminator)]
pub fn macro_type_discriminator(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let out = match input.data {
        Data::Enum(e) => {
            let enum_name = input.ident;
            let enum_display_variants = e.variants.iter().map(|v| {
                let v_string = v.ident.to_string().to_lowercase();
                quote! {
                    #enum_name::#v => { write!(f, #v_string) }
                }
            });
            let fromstr_display_variants = e.variants.iter().map(|v| {
                let v_string = v.ident.to_string().to_lowercase();
                let v = v.ident.clone();
                quote! {
                    #v_string => Ok(#enum_name::#v)
                }
            });

            quote! {
                impl Display for #enum_name {
                    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                        match self {
                            #(#enum_display_variants),*
                        }
                    }
                }

                impl FromStr for #enum_name {
                    type Err = &'static str;
                    fn from_str(s: &str) -> Result<Self, Self::Err> {
                        match s {
                            #(#fromstr_display_variants),*,
                            &_ => Err("Invalid type delimiter")
                        }
                    }
                }
            }
        }
        _ => quote! {}
    };

    out.into()
}
