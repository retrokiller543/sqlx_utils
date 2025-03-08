use crate::error::ErrorExt;
use crate::types::columns::ColumnVal;
use crate::types::condition::Condition;
use crate::types::crate_name;
use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::{format_ident, quote, ToTokens};
use std::collections::HashMap;
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::{parse_quote, TypePath};

/// Represents a logical expression in the WHERE clause.
///
/// This enum handles boolean expressions composed of conditions and
/// logical operators (AND, OR, NOT).
///
/// # Variants
///
/// - `And`: Two expressions combined with AND
/// - `Or`: Two expressions combined with OR
/// - `Not`: Negation of an expression
/// - `Condition`: A basic condition
///
/// # Parsing
///
/// Recursively parses boolean expressions in the format:
/// ```ignore
/// condition AND condition
/// condition OR condition
/// NOT condition
/// condition
/// ```
///
/// # Code Generation
///
/// Expands to methods on the filter struct that implement the corresponding
/// SQL filter operations using the operators defined in the crate:
/// - `AND`: `.and()`
/// - `OR`: `.or()`
/// - `NOT`: `.not()`
pub(crate) enum Expression {
    And(Box<Expression>, Box<Expression>),
    Or(Box<Expression>, Box<Expression>),
    Not(Box<Expression>),
    Group(Box<Expression>),
    Condition(Condition),
}

impl Expression {
    pub fn parse_operator(self, input: ParseStream) -> syn::Result<Self> {
        let mut span = input.span();
        let op: Option<Ident> = input
            .parse()
            .map_err(|err| err.with_context("Failed to parse operator", Some(span)))?;

        match op.map(|i| i.to_string().to_uppercase()) {
            Some(op) if op == *"AND" => {
                span = input.span();
                let right = Self::parse(input).map_err(|err| {
                    err.with_context("Failed to parse right hand side of expression", Some(span))
                })?;
                Ok(Expression::And(Box::new(self), Box::new(right)))
            }
            Some(op) if op == *"OR" => {
                span = input.span();
                let right = Self::parse(input).map_err(|err| {
                    err.with_context("Failed to parse right hand side of expression", Some(span))
                })?;
                Ok(Expression::Or(Box::new(self), Box::new(right)))
            }
            Some(op) if op == *"NOT" => {
                span = input.span();
                let expr = Self::parse(input).map_err(|err| {
                    err.with_context("Failed to parse negated expression", Some(span))
                })?;
                Ok(Expression::Not(Box::new(expr)))
            }
            None => Ok(self),
            Some(op) => Err(syn::Error::new(
                op.span(),
                "Unexpected operator, `{}` expected one of `AND`, `OR`, or `NOT`",
            )),
        }
    }

    pub(crate) fn fields_with_cache<'a>(
        &'a self,
        cache: &mut HashMap<usize, Vec<(&'a Ident, &'a ColumnVal, bool)>>,
    ) -> Vec<(&'a Ident, &'a ColumnVal, bool)> {
        let self_ptr = self as *const Self as usize;

        if let Some(cached) = cache.get(&self_ptr) {
            return cached.clone();
        }

        let mut fields = Vec::new();

        match self {
            Expression::Condition(condition) => fields.push((
                condition.rust_name(),
                &condition.column_type,
                condition.optional,
            )),
            Expression::And(left, right) => {
                fields.extend(left.fields_with_cache(cache));
                fields.extend(right.fields_with_cache(cache));
            }
            Expression::Or(left, right) => {
                fields.extend(left.fields_with_cache(cache));
                fields.extend(right.fields_with_cache(cache));
            }
            Expression::Not(expr) => fields.extend(expr.fields_with_cache(cache)),
            Expression::Group(expr) => fields.extend(expr.fields_with_cache(cache)),
        }

        cache.insert(self_ptr, fields.clone());

        fields
    }
}

impl Parse for Expression {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(syn::token::Paren) {
            let content;
            syn::parenthesized!(content in input);
            let span = content.span();
            let expr = Self::parse(&content)
                .map_err(|err| err.with_context("Failed parse inner Expression", Some(span)))?;

            if !input.is_empty() {
                return expr.parse_operator(input);
            }

            return Ok(Expression::Group(Box::new(expr)));
        }

        // Base condition
        let span = input.span();
        let left =
            Expression::Condition(input.parse().map_err(|err| {
                err.with_context("Failed to parse condition expression", Some(span))
            })?);

        left.parse_operator(input)
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
            Self::Group(e) => quote! { #e },
        };

        tokens.extend(filter_expr);
    }
}
