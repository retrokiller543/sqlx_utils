use crate::types::crate_name;
#[cfg(feature = "try-parse")]
use proc_macro_error2::emit_error;
use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::{ToTokens, quote};
use syn::Token;
use syn::parse::{Parse, ParseStream};

/// Represents SQL comparison operators.
///
/// This enum handles the various operators that can be used in SQL WHERE clauses.
///
/// # Variants
///
/// - `Equals`: `=`
/// - `NotEquals`: `!=` or `NOT =`
/// - `GreaterThan`: `>`
/// - `LessThan`: `<`
/// - `GreaterThanOrEqual`: `>=`
/// - `LessThanOrEqual`: `<=`
/// - `Like`: `LIKE`
/// - `ILike`: `ILIKE`
/// - `In`: `IN`
/// - `NotIn`: `NOT IN`
///
/// # Parsing
///
/// Parses SQL operators from token streams, handling both symbolic operators
/// (`=`, `>`, `<`, etc.) and keyword operators (`LIKE`, `IN`, etc.).
///
/// # Code Generation
///
/// Maps each operator to the corresponding filter function in the crate:
/// - `=` → `equals`
/// - `!=` → `not_equals`
/// - `>` → `greater_than`
/// - `<` → `less_than`
/// - `>=` → `greater_than_or_equal`
/// - `<=` → `less_than_or_equal`
/// - `LIKE` → `like`
/// - `ILIKE` → `i_like`
/// - `IN` → `in_values`
/// - `NOT IN` → `not_in_values`
#[derive(Copy, Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
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

impl SqlOperator {
    /// - `=` → `equals`
    /// - `!=` → `not_equals`
    /// - `>` → `greater_than`
    /// - `<` → `less_than`
    /// - `>=` → `greater_than_or_equal`
    /// - `<=` → `less_than_or_equal`
    /// - `LIKE` → `like`
    /// - `ILIKE` → `i_like`
    /// - `IN` → `in_values`
    /// - `NOT IN` → `not_in_values`
    pub(crate) const SUPPORTED: [&'static str; 10] = [
        "=", "!=", ">", "<", ">=", "<=", "LIKE", "ILIKE", "IN", "NOT IN",
    ];
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
            Ok(match op.to_string().to_uppercase().as_str() {
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
                            Err(syn::Error::new(
                                next.span(),
                                format!("Expected `IN` after `NOT`, found `{}`", next),
                            ))
                        }
                    }
                }
                invalid => Err(syn::Error::new(
                    op.span(),
                    format!("Invalid SQL operator `{}`", invalid),
                )),
            }
            .unwrap_or_else(|error| {
                let supported = Self::SUPPORTED.join(", ");

                #[cfg(not(feature = "try-parse"))]
                proc_macro_error2::abort!(
                    error.span(), "{}", error;
                    help = "Supported operators are: {}", supported;
                );

                #[cfg(feature = "try-parse")]
                emit_error!(
                    error.span(), "{}", error;
                    help = "Supported operators are: {}", supported;
                );

                #[cfg(feature = "try-parse")]
                SqlOperator::Equals
            }))
        }
    }
}

impl ToTokens for SqlOperator {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let crate_name = crate_name();

        match self {
            SqlOperator::Equals => quote! {::#crate_name::filter::equals},
            SqlOperator::NotEquals => quote! {::#crate_name::filter::not_equals},
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
