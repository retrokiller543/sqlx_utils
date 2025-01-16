use crate::types::columns::Columns;
use crate::types::expression::Expression;
use proc_macro2::Ident;
use syn::parse::{Parse, ParseStream};

/// Parses `SELECT * FROM example_table WHERE [conditions]`
#[allow(dead_code)]
pub(crate) struct FilterSql {
    pub(crate) columns: Columns,
    pub(crate) table_name: Ident,
    pub(crate) expr: Expression,
}

impl Parse for FilterSql {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let select = input.parse::<Ident>()?;

        if select.to_string().to_uppercase().as_str() != "SELECT" {
            return Err(input.error("Expected `SELECT`"));
        }

        let columns = input.parse()?;

        let from = input.parse::<Ident>()?;

        if from.to_string().to_uppercase().as_str() != "FROM" {
            return Err(input.error("Expected `FROM`"));
        }

        let table_name: Ident = input.parse()?;

        let where_ident = input.parse::<Ident>()?;

        if where_ident.to_string().to_uppercase().as_str() != "WHERE" {
            return Err(input.error("Expected `WHERE`"));
        }

        let expr = input.parse()?;

        Ok(FilterSql {
            columns,
            table_name,
            expr,
        })
    }
}
