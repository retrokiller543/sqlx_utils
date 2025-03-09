#![allow(async_fn_in_trait)]
#![cfg_attr(all(doc, CHANNEL_NIGHTLY), feature(doc_auto_cfg))]

pub mod error;
pub mod filter;
mod macros;
pub mod pool;
pub mod prelude;
pub mod sqlx;
mod test;
pub mod traits;
pub mod types;
pub mod utils;

pub use error::{Error, Result};
pub use sqlx_utils_macro::sql_filter;
