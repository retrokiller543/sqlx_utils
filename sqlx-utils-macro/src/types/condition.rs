use crate::error::ErrorExt;
use crate::types::columns::ColumnVal;
use crate::types::sql_operator::SqlOperator;
use proc_macro2::Ident;
use syn::parse::{Parse, ParseStream};
use syn::Token;

/// Represents a single condition in the WHERE clause.
///
/// This struct handles individual filter conditions that compare a column
/// to a value using an operator.
///
/// # Fields
///
/// - `column_name`: The name of the database column
/// - `field_alias`: Optional alternative name for the field in the generated struct
/// - `operator`: The SQL operator to use for comparison
/// - `column_type`: The type of the value (Rust type or raw SQL)
/// - `optional`: Whether this condition is optional in the filter
///
/// # Parsing
///
/// Parses conditions in the format:
/// ```ignore
/// [?]column_name [as field_alias] operator value_type
/// ```
///
/// Where:
/// - Optional `?` prefix marks the field as optional
/// - `column_name` is the database column name
/// - Optional `as field_alias` specifies an alternative field name
/// - `operator` is an SQL operator like `=`, `>`, `LIKE`, etc.
/// - `value_type` is either a Rust type or a raw SQL string
///
/// # Code Generation
///
/// Expands to:
/// - A field in the generated struct
/// - A parameter in the constructor (for required fields)
/// - A builder method (for optional fields)
/// - Part of the `apply_filter` implementation
pub(crate) struct Condition {
    pub(crate) column_name: Ident,
    pub(crate) field_alias: Option<Ident>,
    pub(crate) operator: SqlOperator,
    pub(crate) column_type: ColumnVal,
    pub(crate) optional: bool,
}

impl Condition {
    pub(crate) fn rust_name(&self) -> &Ident {
        if let Some(alias) = &self.field_alias {
            alias
        } else {
            &self.column_name
        }
    }
}

impl Parse for Condition {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let optional = input.peek(Token![?]);
        if optional {
            input.parse::<Token![?]>()?;
        }

        let mut span = input.span();
        let column_name = input.parse().map_err(|err| {
            err.with_context(
                "Failed to parse column name, expected a identifier",
                Some(span),
            )
        })?;
        let mut field_alias = None;

        let lookahead = input.lookahead1();

        if lookahead.peek(Token![as]) {
            input.parse::<Token![as]>()?;

            span = input.span();
            field_alias = input.parse().map_err(|err| {
                err.with_context(
                    "Failed to parse field alias, expected a identifier",
                    Some(span),
                )
            })?;
        }

        let operator = input.parse()?;

        span = input.span();
        let column_type = input
            .parse()
            .map_err(|err| err.with_context("Failed to parse Column value", Some(span)))?;

        Ok(Self {
            column_name,
            field_alias,
            operator,
            column_type,
            optional,
        })
    }
}
