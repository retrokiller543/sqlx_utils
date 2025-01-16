use quote::{quote, ToTokens};
use proc_macro2::{Ident, TokenStream as TokenStream2};
use syn::parse::{Parse, ParseStream};
use syn::Token;
use crate::types::crate_name;

pub(crate) enum SqlOperator {
    Equals,
    NotEquals,
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
    Like,
    ILike,
    In,
    NotIn,
}

impl Parse for SqlOperator {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();

        if lookahead.peek(Token![=]) {
            input.parse::<Token![=]>()?;
            Ok(SqlOperator::Equals)
        } else if lookahead.peek(Token![>]) {
            input.parse::<Token![>]>()?;
            if input.peek(Token![=]) {
                input.parse::<Token![=]>()?;
                Ok(SqlOperator::GreaterThanOrEqual)
            } else {
                Ok(SqlOperator::GreaterThan)
            }
        } else if lookahead.peek(Token![<]) {
            input.parse::<Token![<]>()?;
            if input.peek(Token![=]) {
                input.parse::<Token![=]>()?;
                Ok(SqlOperator::LessThanOrEqual)
            } else {
                Ok(SqlOperator::LessThan)
            }
        } else {
            let op: Ident = input.parse()?;
            match op.to_string().to_uppercase().as_str() {
                "LIKE" => Ok(SqlOperator::Like),
                "ILIKE" => Ok(SqlOperator::ILike),
                "IN" => Ok(SqlOperator::In),
                "NOT" => {
                    if input.peek(Token![=]) {
                        input.parse::<Token![=]>()?;
                        Ok(SqlOperator::NotEquals)
                    } else {
                        let next: Ident = input.parse()?;
                        if next.to_string().to_uppercase() == "IN" {
                            Ok(SqlOperator::NotIn)
                        } else {
                            Err(syn::Error::new(next.span(), "Expected 'IN' after 'NOT'"))
                        }
                    }
                }
                _ => Err(syn::Error::new(op.span(), "Invalid SQL operator")),
            }
        }
    }
}

impl ToTokens for SqlOperator {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let crate_name = crate_name();
        
        match self {
            SqlOperator::Equals => quote! {::#crate_name::filter::equals},
            SqlOperator::NotEquals => quote! {::#crate_name::filter::not_equal},
            SqlOperator::GreaterThan => quote! {::#crate_name::filter::greater_than},
            SqlOperator::LessThan => quote! {::#crate_name::filter::less_than},
            SqlOperator::GreaterThanOrEqual => {
                quote! {::#crate_name::filter::greater_than_or_equal}
            }
            SqlOperator::LessThanOrEqual => {
                quote! {::#crate_name::filter::less_than_or_equal}
            }
            SqlOperator::Like => quote! {::#crate_name::filter::like},
            SqlOperator::ILike => quote! {::#crate_name::filter::i_like},
            SqlOperator::In => quote! {::#crate_name::filter::in_values},
            SqlOperator::NotIn => quote! {::#crate_name::filter::not_in_values},
        }
        .to_tokens(tokens)
    }
}