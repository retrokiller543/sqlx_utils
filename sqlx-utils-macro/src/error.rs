#![allow(dead_code)]

use std::fmt::Display;

pub(crate) trait ErrorExt {
    fn with_context(self, context: impl Display, span: Option<proc_macro2::Span>) -> Self;
    fn with_suggestion(self, suggestion: impl Display, span: Option<proc_macro2::Span>) -> Self;
}

impl ErrorExt for syn::Error {
    fn with_context(self, context: impl Display, span: Option<proc_macro2::Span>) -> Self {
        let span = span.unwrap_or_else(proc_macro2::Span::call_site);
        let mut error = self;
        error.combine(syn::Error::new(span, context));
        error
    }

    fn with_suggestion(self, suggestion: impl Display, span: Option<proc_macro2::Span>) -> Self {
        let span = span.unwrap_or_else(proc_macro2::Span::call_site);
        let mut error = self;
        error.combine(syn::Error::new(span, format!("Suggestion: {}", suggestion)));
        error
    }
}

impl<T> ErrorExt for Result<T, syn::Error> {
    fn with_context(self, context: impl Display, span: Option<proc_macro2::Span>) -> Self {
        self.map_err(|error| error.with_context(context, span))
    }

    fn with_suggestion(self, suggestion: impl Display, span: Option<proc_macro2::Span>) -> Self {
        self.map_err(|error| error.with_suggestion(suggestion, span))
    }
}
