use quote::{quote, ToTokens};
use proc_macro2::{Ident, TokenStream as TokenStream2};
use syn_derive::Parse;
use syn::{Attribute, Token, Visibility};
use syn::token::Brace;
use crate::types::columns::ColumnVal;
use crate::types::{crate_name, database_type};
use crate::types::filter_sql::FilterSql;

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
            .filter(|(_, ty, _)| {
                if let ColumnVal::Type(_) = ty {
                    true
                } else {
                    false
                }
            })
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

        let expanded = quote! {
            impl<'args> #crate_name::traits::SqlFilter<'args> for #name {
                #[inline]
                fn apply_filter(self, builder: &mut ::sqlx::QueryBuilder<'args, #db_type>) {
                    #expr.apply_filter(builder);
                }

                #[inline]
                fn should_apply_filter(&self) -> bool {
                    true
                }
            }
        };

        tokens.extend(expanded);
    }
}