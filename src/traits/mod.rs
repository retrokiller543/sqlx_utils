//! Traits defined by this crate
//!
//! All traits are work in progress and might have breaking changes between versions as the project
//! is in early stages. Any feedback on edge cases with trait or lifetime bounds are appreciated as
//! im developing parallel with my other projects that uses this.

//#![deny(missing_docs)]

use crate::mod_def;

mod_def! {
    pub mod model;
    pub mod sql_filter;
    pub mod repository;
}
