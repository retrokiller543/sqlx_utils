use crate::types::Pool;
use std::sync::OnceLock;

pub static DB_POOL: OnceLock<Pool> = OnceLock::new();

#[inline(always)]
pub fn initialize_db_pool(pool: Pool) {
    DB_POOL.set(pool).expect("Failed to set DB_POOL")
}

#[inline(always)]
#[tracing::instrument(level = "debug")]
pub fn get_db_pool() -> &'static Pool {
    DB_POOL.get().expect("DB_POOL is not initialized, please call `initialize_db_pool` before using it, preferably as early as possible in your program!")
}
