use types::filter_table::FilterTable;
use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

const CRATE_NAME_STR: &str = "sqlx_utils";

mod types;

//#[proc_macro_error::proc_macro_error]
#[proc_macro]
pub fn sql_filter(token_stream: TokenStream) -> TokenStream {
    let input = parse_macro_input!(token_stream as FilterTable);

    let expanded = quote! {
        #input
    };

    expanded.into()
}
