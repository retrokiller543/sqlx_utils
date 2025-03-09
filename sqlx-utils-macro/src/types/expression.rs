use crate::types::columns::ColumnVal;
use crate::types::condition::Condition;
use crate::types::crate_name;
#[cfg(feature = "try-parse")]
use proc_macro_error2::emit_error;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::{ToTokens, format_ident, quote};
use std::collections::HashMap;
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::{TypePath, parse_quote};

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
#[derive(Debug)]
pub(crate) enum Expression {
    And(Box<Expression>, Box<Expression>),
    Or(Box<Expression>, Box<Expression>),
    Not(Box<Expression>),
    Group(Box<Expression>),
    Condition {
        condition: Condition,
        #[allow(dead_code)]
        span: Span,
    },
    Empty,
}

impl Expression {
    fn parse_inner(input: ParseStream, start_span: Span) -> syn::Result<Self> {
        if input.peek(syn::token::Paren) {
            let content;
            syn::parenthesized!(content in input);
            let expr = Self::parse_inner(&content, start_span).unwrap_or_else(|e| {
                let message = e.to_string();

                #[cfg(not(feature = "try-parse"))]
                proc_macro_error2::abort!(
                    e.span(),
                    "Failed to parse inner expression: {}",
                    message
                );

                #[cfg(feature = "try-parse")]
                emit_error! {
                    e.span(), "Failed to parse inner expression: {}", message
                }

                #[cfg(feature = "try-parse")]
                Expression::Empty
            });

            #[cfg(feature = "nightly")]
            if let Expression::Condition { span, .. } = &expr {
                proc_macro_error2::emit_warning!(
                    span,
                    "Unnecessary parentheses around simple condition";
                    help = "You can remove these parentheses to simplify your filter"
                );
            }

            if !input.is_empty() {
                return expr.parse_operator(input);
            }

            return Ok(Expression::Group(Box::new(expr)));
        }

        // Base condition
        let condition = input.parse()?;
        let span = start_span.join(input.span()).unwrap_or(start_span);
        let left = Expression::Condition { condition, span };

        left.parse_operator(input)
    }

    pub fn parse_operator(self, input: ParseStream) -> syn::Result<Self> {
        let op: Option<Ident> = input.parse().unwrap_or_else(|err| {
            #[cfg(not(feature = "try-parse"))]
            proc_macro_error2::abort!(err.span(), "Failed to parse operator");

            #[cfg(feature = "try-parse")]
            emit_error!(err.span(), "Failed to parse operator");

            #[cfg(feature = "try-parse")]
            None
        });

        match op.map(|i| i.to_string().to_uppercase()) {
            Some(op) if op == *"AND" => {
                let right = Self::parse(input).unwrap_or(Expression::Empty);
                Ok(Expression::And(Box::new(self), Box::new(right)))
            }
            Some(op) if op == *"OR" => {
                let right = Self::parse(input).unwrap_or(Expression::Empty);
                Ok(Expression::Or(Box::new(self), Box::new(right)))
            }
            Some(op) if op == *"NOT" => {
                let expr = Self::parse(input).unwrap_or(Expression::Empty);
                Ok(Expression::Not(Box::new(expr)))
            }
            None => Ok(self),
            Some(op) => {
                #[cfg(not(feature = "try-parse"))]
                proc_macro_error2::abort!(
                    op.span(),
                    "Unknown operator `{}`, expected one of `AND`, `OR`, or `NOT`",
                    op,
                );

                #[cfg(feature = "try-parse")]
                emit_error!(
                    op.span(),
                    "Unknown operator `{}`, expected one of `AND`, `OR`, or `NOT`",
                    op,
                );

                #[cfg(feature = "try-parse")]
                Ok(Expression::Empty)
            }
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
            Expression::Condition { condition, .. } => fields.push((
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
            _ => {}
        }

        cache.insert(self_ptr, fields.clone());

        fields
    }
}

impl Parse for Expression {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Self::parse_inner(input, input.span())
    }
}

impl ToTokens for Expression {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let filter_expr = match self {
            Expression::Condition { condition: c, .. } => {
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
            _ => TokenStream2::new(),
        };

        tokens.extend(filter_expr);
    }
}
