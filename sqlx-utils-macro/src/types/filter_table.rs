use crate::types::columns::ColumnVal;
use crate::types::filter_sql::FilterSql;
use crate::types::{crate_name, database_type};
use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::{quote, ToTokens};
use syn::token::Brace;
use syn::{parse_quote, Attribute, Token, Visibility};
use syn_derive::Parse;

/// Top-level structure representing a SQL filter definition.
///
/// This struct is the main entry point for parsing the `sql_filter!` macro input.
/// It contains metadata about the generated struct (attributes, visibility, name)
/// and the SQL filter definition.
///
/// # Parsing
///
/// Parses input in the format:
/// ```ignore
/// #[derive(Debug)] // optional attributes
/// pub struct FilterName { // visibility and struct name
///     SELECT * FROM table WHERE // SQL filter definition
///     column = Type AND ...
/// }
/// ```
///
/// # Code Generation
///
/// Expands to:
/// 1. A struct with fields for each condition in the filter
/// 2. A constructor method with required fields as parameters
/// 3. Builder methods for optional fields (those prefixed with `?`)
/// 4. Implementation of the `SqlFilter` trait
///
/// The generated struct implements the `SqlFilter` trait with:
/// - `apply_filter`: Applies the filter conditions to a query builder
/// - `should_apply_filter`: Determines if the filter should be applied (true if
///   all required fields are present and at least one optional field is present)
#[derive(Parse)]
#[allow(dead_code)]
pub(crate) struct FilterTable {
    #[parse(Attribute::parse_outer)]
    pub(crate) meta: Vec<Attribute>,
    pub(crate) vis: Visibility,
    _struct: Token![struct],
    pub(crate) name: Ident,

    #[syn(braced)]
    _brace: Brace,

    #[syn(in = _brace)]
    pub(crate) sql: FilterSql,
}

impl ToTokens for FilterTable {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let crate_name = crate_name();

        let Self {
            meta,
            vis,
            name,
            sql,
            ..
        } = self;

        let fields = sql.expr.fields();
        let token_fields = fields.iter().filter_map(|(name, ty, optional)| {
            if let ColumnVal::Type(ty) = ty {
                if *optional {
                    Some(quote! {#name: Option<#ty>})
                } else {
                    Some(quote! {#name: #ty})
                }
            } else {
                None
            }
        });

        let fields = fields
            .iter()
            .filter(|(_, ty, _)| matches!(ty, ColumnVal::Type(_)))
            .collect::<Vec<_>>();

        let optional_fields = fields
            .iter()
            .filter(|(_, _, optional)| *optional)
            .collect::<Vec<_>>();

        let optional_field_names = optional_fields.iter().map(|(name, _, _)| quote! {#name});

        let optional_field_builder = optional_fields.iter().map(|(name, ty, _)| {
            if let ColumnVal::Type(ty) = ty {
                quote! {
                    #[inline]
                    #vis fn #name(mut self, #name: impl Into<#ty>) -> Self {
                        self.#name = Some(#name.into());
                        self
                    }
                }
            } else {
                quote! {compile_error!("Found Raw type among fields")}
            }
        });

        let req_fields = fields
            .iter()
            .filter(|(_, _, optional)| !*optional)
            .collect::<Vec<_>>();

        let req_fields_fn_input = req_fields.iter().map(|(name, ty, _)| {
            if let ColumnVal::Type(ty) = ty {
                quote! {#name: impl Into<#ty>}
            } else {
                quote! {compile_error!("Found Raw type among fields")}
            }
        });

        let req_fields_into = req_fields.iter().map(|(name, _, _)| {
            quote! {let #name = #name.into();}
        });

        let req_field_names = req_fields.iter().map(|(name, _, _)| quote! {#name});

        let struct_init = if !req_fields.is_empty() {
            quote! {
                Self {
                    #(#req_field_names),*,
                    #(#optional_field_names: None),*
                }
            }
        } else {
            quote! {
                Self {
                    #(#optional_field_names: None),*
                }
            }
        };

        #[cfg(feature = "filter_debug_impl")]
        let mut new_meta = Vec::with_capacity(meta.len());
        #[cfg(feature = "filter_debug_impl")]
        new_meta.extend(meta);
        #[cfg(feature = "filter_debug_impl")]
        let mut meta = new_meta;

        #[cfg(feature = "filter_debug_impl")]
        let debug_meta = parse_quote! {#[derive(Debug)]};
        #[cfg(feature = "filter_debug_impl")]
        meta.push(&debug_meta);


        let struct_def = quote! {
            #(#meta)*
            #vis struct #name {
                #(#token_fields,)*
            }

            impl #name {
                #[inline]
                #vis fn new( #(#req_fields_fn_input),* ) -> Self {
                    #(#req_fields_into)*

                    #struct_init
                }

                #(#optional_field_builder)*
            }
        };

        tokens.extend(struct_def);

        let FilterSql {
            /*columns,
            table_name,*/
            expr,
            ..
        } = sql;

        /*let stmt = quote! {
            SELECT #columns FROM #table_name WHERE
        }.to_string();*/

        let db_type = database_type();

        let should_apply_filter_impl = if !optional_fields.is_empty() {
            let mut impl_tokens = vec![];

            if !req_fields.is_empty() {
                impl_tokens.push(quote! {true});
            }

            for (ident, _, _) in optional_fields {
                impl_tokens.push(quote! {self.#ident.is_some()})
            }

            quote! {#(#impl_tokens)||*}
        } else {
            quote! {true}
        };

        let expanded = quote! {
            impl<'args> #crate_name::traits::SqlFilter<'args> for #name {
                #[inline]
                fn apply_filter(self, builder: &mut ::sqlx::QueryBuilder<'args, #db_type>) {
                    #expr.apply_filter(builder);
                }

                #[inline]
                fn should_apply_filter(&self) -> bool {
                    #should_apply_filter_impl
                }
            }
        };

        tokens.extend(expanded);
    }
}
