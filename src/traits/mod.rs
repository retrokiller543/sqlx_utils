//! Traits defined by this crate

#![deny(missing_docs)]

use crate::mod_def;

mod_def! {
    pub mod model;
    pub mod sql_filter;
    pub mod repository;
}
