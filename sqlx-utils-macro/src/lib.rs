use crate::filter::expand;
use derive::model::DeriveModel;
use proc_macro::TokenStream;
use quote::ToTokens;

const CRATE_NAME_STR: &str = "sqlx_utils";

mod derive;
mod error;
mod filter;
mod types;

/// Creates a type-safe SQL filter struct with builder methods using SQL-like syntax.
///
/// This macro generates a struct that implements the `SqlFilter` trait, making it
/// suitable for use with repository query methods that accept SQL filters.
///
/// # Syntax
///
/// ```ignore
/// sql_filter! {
///     [attributes]
///     visibility struct StructName {
///         SELECT columns FROM table_name WHERE
///         condition [AND|OR] condition ...
///     }
/// }
/// ```
///
/// ## Columns
///
/// You can select all columns with `*` or specify individual columns:
/// - `SELECT * FROM ...`: Select all columns
/// - `SELECT col1, col2 as alias FROM ...`: Select specific columns with optional aliases
///
/// ## Conditions
///
/// Conditions use SQL-like syntax:
/// ```ignore
/// [?]column_name [as field_name] OPERATOR type
/// ```
///
/// - Optional `?` prefix marks the field as optional in the filter
/// - `column_name` is the database column name
/// - Optional `as field_name` to use a different field name in the generated struct
/// - `OPERATOR` is one of: `=`, `!=`, `>`, `<`, `>=`, `<=`, `LIKE`, `ILIKE`, `IN`, `NOT IN`
/// - `type` is either a Rust type (e.g., `i32`, `String`) or a raw SQL string literal
///
/// Conditions can be combined with logical operators `AND`, `OR`, and `NOT`.
///
/// # Generated Code
///
/// The macro generates:
/// 1. A struct with fields for each condition
/// 2. A constructor method for required fields
/// 3. Builder methods for optional fields
/// 4. Implementation of the `SqlFilter` trait
///
/// # Examples
///
/// ## Basic Filter
///
/// ```rust,ignore
/// # use sqlx_utils_macro::sql_filter;
/// sql_filter! {
///     pub struct UserFilter {
///         SELECT * FROM users WHERE
///         id = i32
///     }
/// }
///
/// // Usage:
/// let filter = UserFilter::new(1);
/// ```
///
/// ## Filter with Optional Fields
///
/// ```rust,ignore
/// # use sqlx_utils_macro::sql_filter;
/// sql_filter! {
///     pub struct UserFilter {
///         SELECT * FROM users WHERE
///         ?name LIKE String AND
///         ?age >= i32
///     }
/// }
///
/// // Usage:
/// let filter = UserFilter::new()
///     .name("John%")
///     .age(18);
/// ```
///
/// ## Filter with Raw SQL
///
/// ```rust,ignore
/// # use sqlx_utils_macro::sql_filter;
/// sql_filter! {
///     pub struct UserFilter {
///         SELECT * FROM users WHERE
///         created_at > "NOW() - INTERVAL '1 day'"
///     }
/// }
/// ```
///
/// ## Complex Filter
///
/// ```rust,ignore
/// # use sqlx_utils_macro::sql_filter;
/// sql_filter! {
///     pub struct OrderFilter {
///         SELECT id, total, name as customer_name FROM orders WHERE
///         id = i32 OR
///         (total > f64 AND ?customer_id = i32)
///     }
/// }
/// ```
#[proc_macro_error2::proc_macro_error]
#[proc_macro]
pub fn sql_filter(token_stream: TokenStream) -> TokenStream {
    expand(token_stream)
}

#[proc_macro_error2::proc_macro_error]
#[proc_macro_derive(Model, attributes(model))]
pub fn derive_model(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    println!("Parsed input: {input:#?}");
    
    match DeriveModel::new(input) {
        Ok(model) => model.to_token_stream().into(),
        Err(err) => err.into(),
    }
}
