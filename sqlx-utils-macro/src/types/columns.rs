use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::{LitStr, Token, Type};

/// Parses either `*` or `a, b, c as c_example`
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

pub(crate) enum ColumnVal {
    Type(Type),
    Raw(LitStr),
}

impl Parse for ColumnVal {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();

        if lookahead.peek(LitStr) {
            Ok(Self::Raw(input.parse()?))
        } else {
            Ok(Self::Type(input.parse()?))
        }
    }
}
