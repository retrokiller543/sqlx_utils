#![allow(async_fn_in_trait)]

pub mod macros;
pub mod utils;
pub mod filter;
pub mod traits;
pub mod error;
pub mod types;
pub mod pool;

pub use sqlx_utils_macro::sql_filter;
pub use error::{Result, Error};
