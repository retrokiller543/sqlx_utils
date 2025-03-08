use crate::error::ErrorExt;
use crate::types::columns::Columns;
use crate::types::expression::Expression;
use proc_macro2::Ident;
use syn::parse::{Parse, ParseStream};
use syn::Error;

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
        let mut span = input.span();
        let select = input.parse::<Ident>()?;

        if select.to_string().to_uppercase().as_str() != "SELECT" {
            return Err(Error::new(
                span,
                format!("Expected `SELECT`, found `{}`", select),
            ));
        }

        let columns = input.parse()?;

        span = input.span();
        let from = input.parse::<Ident>()?;

        if from.to_string().to_uppercase().as_str() != "FROM" {
            return Err(Error::new(
                span,
                format!("Expected `FROM`, found `{}`", from),
            ));
        }

        span = input.span();
        let table_name: Ident = input.parse().with_context(
            "Failed to parse table name, expected an identifier",
            Some(span),
        )?;

        span = input.span();
        let where_ident = input
            .parse::<Ident>()
            .with_context("Expected an identifier after the table name", Some(span))?;

        if where_ident.to_string().to_uppercase().as_str() != "WHERE" {
            return Err(Error::new(
                span,
                format!("Expected `WHERE`, found `{}`", where_ident),
            ));
        }

        let expr = input.parse()?;

        Ok(FilterSql {
            columns,
            table_name,
            expr,
        })
    }
}
