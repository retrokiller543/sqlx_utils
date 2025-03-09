use crate::types::filter_table::FilterTable;
use proc_macro::TokenStream;
use proc_macro_error2::abort;
use quote::ToTokens;

pub(crate) fn expand(input: TokenStream) -> TokenStream {
    parse_and_validate(input).to_token_stream().into()
}

fn parse_and_validate(input: TokenStream) -> FilterTable {
    let res: Result<FilterTable, syn::Error> = syn::parse(input);

    match res {
        Ok(table) => table,
        Err(err) => {
            abort!(err);
        }
    }
}
