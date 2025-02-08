use crate::types::columns::ColumnVal;
use crate::types::condition::Condition;
use crate::types::crate_name;
use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::{format_ident, quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::{parse_quote, TypePath};

pub(crate) enum Expression {
    And(Box<Expression>, Box<Expression>),
    Or(Box<Expression>, Box<Expression>),
    Not(Box<Expression>),
    Condition(Condition),
}

impl Expression {
    pub(crate) fn fields(&self) -> Vec<(&Ident, &ColumnVal, bool)> {
        let mut fields = Vec::new();

        match self {
            Expression::Condition(condition) => fields.push((
                condition.rust_name(),
                &condition.column_type,
                condition.optional,
            )),
            Expression::And(left, right) => {
                fields.extend(left.fields());
                fields.extend(right.fields());
            }
            Expression::Or(left, right) => {
                fields.extend(left.fields());
                fields.extend(right.fields());
            }
            Expression::Not(expr) => fields.extend(expr.fields()),
        }

        fields
    }
}

impl Parse for Expression {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Base condition
        let left = Expression::Condition(input.parse()?);

        let op: Option<Ident> = input.parse()?;
        match op.map(|i| i.to_string().to_uppercase()) {
            Some(op) if op == *"AND" => {
                let right = Self::parse(input)?;
                Ok(Expression::And(Box::new(left), Box::new(right)))
            }
            Some(op) if op == *"OR" => {
                let right = Self::parse(input)?;
                Ok(Expression::Or(Box::new(left), Box::new(right)))
            }
            Some(op) if op == *"NOT" => {
                let expr = Self::parse(input)?;
                Ok(Expression::Not(Box::new(expr)))
            }
            None => Ok(left),
            Some(op) => Err(syn::Error::new(op.span(), "Unexpected operator")),
        }
    }
}

impl ToTokens for Expression {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let filter_expr = match self {
            Expression::Condition(c) => {
                let crate_name = crate_name();

                let operator = &c.operator;
                let column = &c.column_name;

                if let ColumnVal::Raw(lit) = &c.column_type {
                    let path: TypePath = parse_quote! {#operator};
                    let seg = path.path.segments.last().unwrap();
                    let ident = &seg.ident;
                    let new_ident = format_ident!("{}_raw", ident);

                    quote! { ::#crate_name::filter::#new_ident(stringify!(#column), ::#crate_name::filter::Raw(#lit)) }
                } else {
                    let rust_name = c.rust_name();

                    if c.optional {
                        quote! { #operator(stringify!(#column), self.#rust_name) }
                    } else {
                        quote! { #operator(stringify!(#column), Some(self.#rust_name)) }
                    }
                }
            }
            Expression::And(l, r) => quote! { #l.and(#r) },
            Expression::Or(l, r) => quote! { #l.or(#r) },
            Expression::Not(e) => quote! { #e.not() },
        };

        tokens.extend(filter_expr);
    }
}
