use crate::CRATE_NAME_STR;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

pub(crate) mod columns;
pub(crate) mod condition;
pub(crate) mod expression;
pub(crate) mod filter_sql;
pub(crate) mod filter_table;
pub(crate) mod sql_operator;

#[inline]
fn crate_name() -> Ident {
    Ident::new(CRATE_NAME_STR, Span::call_site())
}

#[inline]
fn database_type() -> TokenStream {
    if cfg!(feature = "sqlite") {
        quote! {::sqlx::Sqlite}
    } else if cfg!(feature = "mysql") {
        quote! {::sqlx::MySql}
    } else if cfg!(feature = "postgres") {
        quote! {::sqlx::Postgres}
    } else {
        quote! {::sqlx::Any}
    }
}

/*impl SqlOperator {
    fn as_str(&self) -> &'static str {
        match self {
            SqlOperator::Equals => "=",
            SqlOperator::NotEquals => "!=",
            SqlOperator::GreaterThan => ">",
            SqlOperator::LessThan => "<",
            SqlOperator::GreaterThanOrEqual => ">=",
            SqlOperator::LessThanOrEqual => "<=",
            SqlOperator::Like => "LIKE",
            SqlOperator::ILike => "ILIKE",
            SqlOperator::In => "IN",
            SqlOperator::NotIn => "NOT IN",
        }
    }
}
*/
