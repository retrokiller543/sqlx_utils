use crate::types::columns::ColumnVal;
use crate::types::filter_sql::FilterSql;
use crate::types::{crate_name, database_type};
use proc_macro2::{Ident, TokenStream as TokenStream2};
use proc_macro_error2::abort_call_site;
use quote::{ToTokens, quote};
use syn::parse::ParseStream;
use std::collections::HashMap;
use syn::token::Brace;
use syn::{parse_quote, Attribute, Token, Type, Visibility};
use syn_derive::Parse;
use std::fmt::Write;

use super::columns::Columns;

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
#[derive(Parse, Debug)]
#[allow(dead_code)]
pub(crate) struct FilterTable {
    #[parse(Attribute::parse_outer)]
    pub(crate) meta: Vec<Attribute>,
    pub(crate) vis: Visibility,
    _struct: Token![struct],
    pub(crate) name: Ident,

    #[parse(RepositoryIdent::parse_repository_ident)]
    pub(crate) repo: Option<RepositoryIdent>,

    #[syn(braced)]
    _brace: Brace,

    #[syn(in = _brace)]
    pub(crate) sql: FilterSql,
}

#[derive(Parse, Debug)]
#[allow(dead_code)]
pub(crate) struct RepositoryIdent {
    lt_token: Token![<],
    pub(crate) repo_type: Type,
    gt_token: Token![>]
}

impl RepositoryIdent {
    pub(crate) fn parse_repository_ident(input: ParseStream) -> syn::Result<Option<Self>> {
        if input.peek(Token![<]) {
            Ok(Some(input.parse()?))
        } else {
            Ok(None)
        }
    }

    pub(crate) fn expand_repo_impl(&self, crate_name: &Ident, #[cfg(not(feature = "filter-blanket-impl"))] name: &Ident, sql: &FilterSql, token_stream: &mut TokenStream2) {
        let FilterSql { columns, table_name, .. } = sql;

        let mut query_str = String::from("SELECT");

        match columns {
            Columns::All => write!(query_str, " *").unwrap_or_else(|_| {
                abort_call_site!(
                    "Failed to write column into string, please rapport this as an issue!"
                )
            }),
            Columns::Defined(cols) => {
                let columns = cols.iter().cloned().map(|(name, alias)| {
                    if name != alias {
                        format!("{name} as {alias}")
                    } else {
                        name
                    }
                }).collect::<Vec<_>>().join(", ");
                write!(query_str, " {columns}").unwrap_or_else(|_| {
                    abort_call_site!(
                        "Failed to write column into string, please rapport this as an issue!"
                    )
                })
            }
        }

        write!(query_str, " FROM {} ", table_name).unwrap_or_else(|_| {
            abort_call_site!(
                "Failed to write column into string, please rapport this as an issue!"
            )
        });

        //M: Model + for<'r> FromRow<'r, <Database as DatabaseTrait>::Row> + Send + Unpin,
        let repo_ident = &self.repo_type;


        let expanded_filter_repo = quote! {
            impl<M> ::#crate_name::traits::FilterRepository<M> for #repo_ident
            where
                M: ::#crate_name::traits::Model + for<'r> ::#crate_name::sqlx::FromRow<'r, <::#crate_name::types::Database as ::#crate_name::sqlx::Database>::Row> + ::core::marker::Send + ::core::marker::Unpin,
                #repo_ident: ::#crate_name::traits::Repository<M>
            {
                fn filter_query_builder<'args>() -> ::#crate_name::sqlx::QueryBuilder<'args, ::#crate_name::types::Database> {
                    ::#crate_name::sqlx::QueryBuilder::new(#query_str)
                }
            }
        };

        token_stream.extend(expanded_filter_repo);

        #[cfg(not(feature = "filter-blanket-impl"))]
        {
            let expanded_ext_filter = quote! {
                impl<M> ::#crate_name::traits::FilterRepositoryExt<M, #name> for #repo_ident
                where
                    M: ::#crate_name::traits::Model + for<'r> ::#crate_name::sqlx::FromRow<'r, <::#crate_name::types::Database as ::#crate_name::sqlx::Database>::Row> + ::core::marker::Send + ::core::marker::Unpin,
                    #repo_ident: ::#crate_name::traits::Repository<M>
                {}
            };

            token_stream.extend(expanded_ext_filter);
        }
    }
}

impl ToTokens for FilterTable {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let crate_name = crate_name();

        let Self {
            meta,
            vis,
            name,
            sql,
            repo,
            ..
        } = self;

        let mut cache = HashMap::new();
        let fields = sql.expr.fields_with_cache(&mut cache);
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
            expr,
            ..
        } = sql;

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
                fn apply_filter(self, builder: &mut ::sqlx_utils::types::QueryBuilder<'args, #db_type>) {
                    #expr.apply_filter(builder);
                }

                #[inline]
                fn should_apply_filter(&self) -> bool {
                    #should_apply_filter_impl
                }
            }
        };

        tokens.extend(expanded);

        if let Some(repo) = repo {
            #[cfg(not(feature = "filter-blanket-impl"))]
            repo.expand_repo_impl(&crate_name, name, sql, tokens);
            #[cfg(feature = "filter-blanket-impl")]
            repo.expand_repo_impl(&crate_name, sql, tokens);
        }
    }
}
