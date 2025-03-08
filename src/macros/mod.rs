mod db;
mod repository;

/// Creates a new database repository, either just creates a basic new type and statics to interact
/// with the main database pool.
///
/// If a database model is provided it will also try to implement the [`crate::traits::Repository`] trait.
///
/// For non ZST repositories it will implement `Deref`, `Borrow`, and `AsRef` to get the inner pool.
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
///         Some(self.id.clone())
///     }
/// }
///
/// repository!{
///     PersonRepository<Person>;
/// }
/// ```
///
/// # Zero Sized Type Repository
///
/// It is possible to make a repository that is zero sized by never storing the reference to the database pool,
/// this will add a slight cost however, whenever we want to use the [`pool()`](crate::traits::Repository::pool)
/// method we now need to access the [DB_POOL](crate::pool::DB_POOL) static via the [`get_db_pool()`](crate::pool::get_db_pool).
/// This cost however is tiny and in most cases not an issue as it will be overshadowed by the actual database request.
///
/// # Example of a ZST Repository
///
/// ```
/// # use sqlx_utils::repository;
/// # use sqlx_utils::traits::Model;
///
///
/// # struct Person {
/// #     id: String,
/// #     name: String
/// # }
///
/// # impl Model for Person {
/// #     type Id = String;
/// #
/// #     fn get_id(&self) -> Option<Self::Id> {
/// #         Some(self.id.clone())
/// #     }
/// # }
/// #
/// repository!{
///     !zst
///     PersonRepository<Person>; // The generated type `PersonRepository` will now have size of 0
/// }
/// ```
#[macro_export]
macro_rules! repository {
    {
        $( #[$meta:meta] )*
        $vis:vis $ident:ident;
    } => {
        $(#[$meta])*
        #[derive(Clone, Copy, Debug)]
        $vis struct $ident {
            /// Static Reference to the global database pool [`DB_POOL`](::sqlx_utils::pool::DB_POOL)
            pool: &'static $crate::types::Pool,
        }

        impl ::core::ops::Deref for $ident {
            type Target = $crate::types::Pool;

            fn deref(&self) -> &Self::Target {
                &self.pool
            }
        }

        impl ::core::borrow::Borrow<$crate::types::Pool> for $ident {
            fn borrow(&self) -> &$crate::types::Pool {
                &self.pool
            }
        }

        impl ::core::convert::AsRef<$crate::types::Pool> for $ident {
            fn as_ref(&self) -> &$crate::types::Pool {
                &self.pool
            }
        }

        $crate::static_repo!($vis $ident;);

        impl $ident {
            #[inline(always)]
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
        $crate::repository!(!inner $(#[$meta])* $vis $ident<$model>; $($tokens)*);
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
    };

    {
        !zst
        $( #[$meta:meta] )*
        $vis:vis $ident:ident;
    } => {
        $(#[$meta])*
        #[derive(Clone, Copy, Debug)]
        $vis struct $ident;

        $crate::static_repo!(!zst $vis $ident;);

        impl $ident {
            #[inline(always)]
            $vis const fn new() -> Self {
                Self
            }
        }
    };

    {
        !zst
        $( #[$meta:meta] )*
        $vis:vis $ident:ident<$model:ty>;
    } => {
        $crate::repository!(
            !zst
            !inner
            $(#[$meta])*
            $vis $ident<$model>;
        );
    };

    {
        !zst
        $( #[$meta:meta] )*
        $vis:vis $ident:ident<$model:ty>;

        $($tokens:tt)*
    } => {
        $crate::repository!(!zst !inner $(#[$meta])* $vis $ident<$model>; $($tokens)*);
    };

    {
        !zst !inner
        $( #[$meta:meta] )*
        $vis:vis $ident:ident<$model:ty>;

        $($tokens:tt)*
    } => {
        $crate::repository!(!zst $(#[$meta])* $vis $ident;);

        impl $crate::traits::Repository<$model> for $ident {
            #[inline]
            fn pool(&self) -> & $crate::types::Pool {
                $crate::pool::get_db_pool()
            }
            $($tokens)*
        }
    };
}
