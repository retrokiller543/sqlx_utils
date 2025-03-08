use crate::mod_def;

mod_def! {!export
    pub(crate) mod query;
    pub(crate) mod pool;
    pub(crate) mod db;
}

macro_rules! db_type {
    ($vis:vis type $ident:ident = [$any_ty:ty, $pg_ty:ty, $mysql_ty:ty, $sqlite_ty:ty]) => {
        #[cfg(feature = "any")]
        $vis type $ident = $any_ty;

        #[cfg(all(feature = "postgres", not(any(feature = "sqlite", feature = "mysql", feature = "any"))))]
        $vis type $ident = $pg_ty;

        #[cfg(all(feature = "mysql", not(any(feature = "sqlite", feature = "any", feature = "postgres"))))]
        $vis type $ident = $mysql_ty;

        #[cfg(all(feature = "sqlite", not(any(feature = "any", feature = "mysql", feature = "postgres"))))]
        $vis type $ident = $sqlite_ty;
    };
}
pub(crate) use db_type;
