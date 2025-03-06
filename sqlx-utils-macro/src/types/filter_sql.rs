use crate::types::columns::Columns;
use crate::types::expression::Expression;
use proc_macro2::Ident;
use syn::parse::{Parse, ParseStream};

/// Represents the SQL query portion of a filter definition.
///
/// This struct parses the SQL-like syntax within the filter definition,
/// extracting the columns to select, table name, and filter expression.
///
/// # Parsing
///
/// Parses input in the format:
/// ```ignore
/// SELECT columns FROM table_name WHERE expression
/// ```
///
/// Where:
/// - `columns` can be `*` or a comma-separated list of column names with optional aliases
/// - `table_name` is the name of the database table
/// - `expression` is a boolean expression combining filter conditions
///
/// # Fields
///
/// - `columns`: The columns to select (either all columns or specific ones)
/// - `table_name`: The name of the database table
/// - `expr`: The parsed filter expression
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
