use crate::types::filter_table::FilterTable;
use proc_macro::TokenStream;
use proc_macro_error2::abort_if_dirty;
use quote::ToTokens;

pub(crate) fn expand(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as FilterTable);

    abort_if_dirty();

    input.to_token_stream().into()
}
