use proc_macro2::Ident;
use syn::parse::{Parse, ParseStream};
use syn::Token;
use crate::types::sql_operator::SqlOperator;
use crate::types::columns::ColumnVal;

/// Parses `example_col LIKE String`
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

        let column_name = input.parse()?;
        let mut field_alias = None;

        let lookahead = input.lookahead1();

        if lookahead.peek(Token![as]) {
            input.parse::<Token![as]>()?;
            field_alias = input.parse()?;
        }

        let operator = input.parse()?;
        let column_type = input.parse()?;

        Ok(Self {
            column_name,
            field_alias,
            operator,
            column_type,
            optional,
        })
    }
}