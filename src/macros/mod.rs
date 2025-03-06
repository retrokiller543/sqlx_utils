mod db;
mod repository;

/// Creates a new database repository, either just creates a basic new type and statics to interact
/// with the main database pool.
///
/// If a database model is provided it will also try to implement the [`crate::traits::Repository`] trait.
///
/// # Examples
///
/// ```
/// use sqlx_utils::repository;
/// use sqlx_utils::traits::Model;
///
///
/// struct Person {
///     id: String,
///     name: String
/// }
///
/// impl Model for Person {
///     type Id = String;
///
///     fn get_id(&self) -> Option<Self::Id> {
///         Some(self.id)
///     }
/// }
///
/// repository!{
///     PersonRepository<Person>;
/// }
/// ```
///
#[macro_export]
macro_rules! repository {
    {
        $( #[$meta:meta] )*
        $vis:vis $ident:ident;
    } => {
        $(#[$meta])*
        #[derive(Clone, Copy, Debug)]
        $vis struct $ident {
            pool: &'static $crate::types::Pool,
        }

        $crate::static_repo!($vis $ident;);

        impl $ident {
            $vis fn new() -> Self {
                let pool = $crate::pool::get_db_pool();

                Self {
                    pool
                }
            }
        }
    };

    {
        $( #[$meta:meta] )*
        $vis:vis $ident:ident<$model:ty>;
    } => {
        $crate::repository!(
            !inner
            $(#[$meta])*
            $vis $ident<$model>;
        );
    };

    {
        $( #[$meta:meta] )*
        $vis:vis $ident:ident<$model:ty>;

        $($tokens:tt)*
    } => {
        $crate::repository!(!inner $(#[$meta])* $vis $ident; $($tokens)*);
    };

    {
        !inner
        $( #[$meta:meta] )*
        $vis:vis $ident:ident<$model:ty>;

        $($tokens:tt)*
    } => {
        $crate::repository!($(#[$meta])* $vis $ident;);

        impl $crate::traits::Repository<$model> for $ident {
            #[inline]
            fn pool(&self) -> & $crate::types::Pool {
                self.pool
            }
            $($tokens)*
        }
    }
}
