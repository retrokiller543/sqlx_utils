#[cfg(feature = "try-parse")]
use proc_macro_error2::emit_error;
use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::{ToTokens, quote};
use syn::parse::{Parse, ParseStream};
#[cfg(feature = "try-parse")]
use syn::parse_quote_spanned;
use syn::{LitStr, Token, Type};

/// Represents column selection in an SQL query.
///
/// This enum handles both the `SELECT *` case and the case where
/// specific columns are selected, potentially with aliases.
///
/// # Variants
///
/// - `All`: Represents `SELECT *`
/// - `Defined`: Represents `SELECT col1, col2 as alias2, ...`
///
/// # Parsing
///
/// Parses input in either format:
/// - `*`: All columns
/// - `col1, col2 as alias2, ...`: Specific columns with optional aliases
///
/// # Code Generation
///
/// Expands to SQL column selectors in the generated query.
#[cfg_attr(debug_assertions, derive(Debug))]
pub(crate) enum Columns {
    All,
    Defined(Vec<(String, String)>),
}

impl ToTokens for Columns {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let expanded = match self {
            Columns::All => quote! {"*"},
            Columns::Defined(cols) => {
                let columns = cols
                    .iter()
                    .map(|(name, alias)| quote! {#name as #alias}.to_string())
                    .collect::<Vec<_>>()
                    .join(",");

                quote! {#columns}
            }
        };

        tokens.extend(expanded);
    }
}

impl Parse for Columns {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(Token![*]) {
            input.parse::<Token![*]>()?;
            Ok(Columns::All)
        } else {
            let mut columns = Vec::new();
            while !input.is_empty() {
                let name: Ident = input.parse()?;
                let alias = if input.peek(Token![as]) {
                    input.parse::<Token![as]>()?;
                    let alias: Ident = input.parse()?;
                    alias.to_string()
                } else {
                    name.to_string()
                };
                columns.push((name.to_string(), alias));

                if !input.peek(Token![,]) {
                    break;
                }
                input.parse::<Token![,]>()?;
            }
            Ok(Columns::Defined(columns))
        }
    }
}

/// Represents a value type in a condition.
///
/// This enum handles both Rust types and raw SQL expressions.
///
/// # Variants
///
/// - `Type`: A Rust type like `i32` or `String`
/// - `Raw`: A raw SQL string literal for direct inclusion in the query
///
/// # Parsing
///
/// - `Type`: Parses any valid Rust type
/// - `Raw`: Parses a string literal enclosed in quotes
///
/// # Usage in Filters
///
/// Used to specify the expected type of a filter field or to include
/// raw SQL expressions directly in the query.
#[cfg_attr(feature = "debug", derive(Debug))]
pub(crate) enum ColumnVal {
    Type(Box<Type>),
    Raw(LitStr),
}

impl Parse for ColumnVal {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();

        if lookahead.peek(LitStr) {
            Ok(Self::Raw(input.parse()?))
        } else {
            Ok(Self::Type(input.parse().unwrap_or_else(|err| {
                let span = err.span();

                #[cfg(not(feature = "try-parse"))]
                proc_macro_error2::abort!(
                    span, "Expected column value to be either a literal string or a Type";
                    help = "See https://docs.rs/syn/2.0.99/syn/enum.Type.html for supported types."
                );

                #[cfg(feature = "try-parse")]
                emit_error!(
                    span, "Expected column value to be either a literal string or a Type";
                    help = "See https://docs.rs/syn/2.0.99/syn/enum.Type.html for supported types."
                );

                #[cfg(feature = "try-parse")]
                parse_quote_spanned! { span=> ()}
            })))
        }
    }
}
