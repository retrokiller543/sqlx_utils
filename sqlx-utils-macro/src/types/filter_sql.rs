use crate::types::columns::Columns;
use crate::types::expression::Expression;
use proc_macro_error2::abort;
#[cfg(feature = "try-parse")]
use proc_macro_error2::emit_error;
use proc_macro2::Ident;
use syn::Error;
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
#[cfg_attr(feature = "debug", derive(Debug))]
pub(crate) struct FilterSql {
    pub(crate) columns: Columns,
    pub(crate) table_name: Ident,
    pub(crate) expr: Expression,
}

impl Parse for FilterSql {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let select = match input.parse::<Ident>() {
            Ok(ident) => ident,
            Err(e) => {
                abort!(
                    e.span(),
                    "Expected SQL query to start with `SELECT` Identifier"
                );
            }
        };

        let select_str = select.to_string();

        if select_str.to_uppercase() != "SELECT" {
            abort!(
                select.span(),
                "Expected `SELECT` at the beginning of SQL filter, got `{}` instead", select_str;
                help = "SQL filter must start with `SELECT` followed by columns, table and WHERE clause";
                note = "It is case insensitive so `select` will also work."
            );
        }

        let columns = input.parse()?;

        let from = input.parse::<Ident>()?;

        if from.to_string().to_uppercase().as_str() != "FROM" {
            return Err(Error::new(
                from.span(),
                format!("Expected `FROM`, found `{}`", from),
            ));
        }

        let table_name: Ident = input.parse().unwrap_or_else(|e| {
            let span = e.span();

            #[cfg(not(feature = "try-parse"))]
            proc_macro_error2::abort!(
                span, "Failed to parse table name";
                help = "The table name can be any identifier not reserved by rust or the keyword `WHERE`";
            );

            #[cfg(feature = "try-parse")]
            emit_error!(
                span, "Failed to parse table name";
                help = "The table name can be any identifier not reserved by rust or the keyword `WHERE`";
            );

            #[cfg(feature = "try-parse")]
            Ident::new("__err__", span)
        });

        #[cfg(feature = "try-parse")]
        let mut table_name_err = false;

        #[cfg(not(feature = "try-parse"))]
        let table_name_err = false;
        if table_name.to_string().to_uppercase().as_str() == "WHERE" {
            #[cfg(not(feature = "try-parse"))]
            proc_macro_error2::abort!(
                table_name, "The keyword `WHERE` is reserved and cant be used as a table name";
                help = "Any identifier is allowed in this location except for `WHERE`";
            );

            #[cfg(feature = "try-parse")]
            {
                emit_error!(
                    table_name, "The keyword `WHERE` is reserved and cant be used as a table name";
                    help = "Any identifier is allowed in this location except for `WHERE`";
                );
                table_name_err = true;
            }
        }

        if !table_name_err {
            let where_ident = input.parse::<Ident>().unwrap_or_else(|err| {
                let span = err.span();

                #[cfg(not(feature = "try-parse"))]
                proc_macro_error2::abort!(span, "Failed to parse `WHERE`");
                #[cfg(feature = "try-parse")]
                emit_error!(span, "Failed to parse `WHERE`");
                #[cfg(feature = "try-parse")]
                Ident::new("__err__", span)
            });

            let where_str = where_ident.to_string();

            if where_str.to_uppercase().as_str() != "WHERE" {
                #[cfg(not(feature = "try-parse"))]
                proc_macro_error2::abort!(
                    where_ident,
                    "Expected `WHERE` but instead found `{}`",
                    where_str
                );

                #[cfg(feature = "try-parse")]
                emit_error! {
                    where_ident, "Expected `WHERE` but instead found `{}`", where_str
                }
            }
        }

        let expr = input.parse()?;

        Ok(FilterSql {
            columns,
            table_name,
            expr,
        })
    }
}
