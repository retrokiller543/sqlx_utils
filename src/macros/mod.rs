pub mod db;

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
            $(#[$meta])*
            $vis $ident<$model>;

            #[inline]
            fn insert_one(_model: &$model) -> $crate::types::Query {
                unimplemented!("Insert is not implemented for this repository");
            }

            #[inline]
            fn update_one(_model: &$model) -> $crate::types::Query {
                unimplemented!("Update is not implemented for this repository");
            }

            #[inline]
            fn delete_one_by_id(_model: &<$model as Model>::Id) -> $crate::types::Query {
                unimplemented!("Delete is not implemented for this repository");
            }
        );
    };

    {
        $( #[$meta:meta] )*
        $vis:vis $ident:ident<$model:ty>;

        insert_one($model_name:ident) $block:block;

        $($tokens:tt)*
    } => {
        $crate::repository!(
            $(#[$meta])*
            $vis $ident<$model>;

            #[inline]
            fn insert_one($model_name: &$model) -> $crate::types::Query $block

            #[inline]
            fn update_one(_model: &$model) -> $crate::types::Query {
                unimplemented!("Update is not implemented for this repository");
            }

            #[inline]
            fn delete_one_by_id(_model: &<$model as Model>::Id) -> $crate::types::Query {
                unimplemented!("Delete is not implemented for this repository");
            }

            $($tokens)*
        );
    };

    {
        $( #[$meta:meta] )*
        $vis:vis $ident:ident<$model:ty>;

        update_one($model_name:ident) $block:block;

        $($tokens:tt)*
    } => {
        $crate::repository!(
            $(#[$meta])*
            $vis $ident<$model>;

            #[inline]
            fn insert_one(_model: &$model) -> $crate::types::Query {
                unimplemented!("Insert is not implemented for this repository");
            }

            #[inline]
            fn update_one($model_name: &$model) -> $crate::types::Query $block

            #[inline]
            fn delete_one_by_id(_model: &<$model as Model>::Id) -> $crate::types::Query {
                unimplemented!("Delete is not implemented for this repository");
            }

            $($tokens)*
        );
    };

    {
        $( #[$meta:meta] )*
        $vis:vis $ident:ident<$model:ty>;

        delete_one_by_id($id_name:ident) $block:block;

        $($tokens:tt)*
    } => {
        $crate::repository!(
            $(#[$meta])*
            $vis $ident<$model>;

            #[inline]
            fn insert_one(_model: &$model) -> $crate::types::Query {
                unimplemented!("Insert is not implemented for this repository");
            }

            #[inline]
            fn update_one(_model: &$model) -> $crate::types::Query {
                unimplemented!("Update is not implemented for this repository");
            }

            #[inline]
            fn delete_one_by_id($id_name: &<$model as Model>::Id) -> $crate::types::Query $block

            $($tokens)*
        );
    };

    {
        $( #[$meta:meta] )*
        $vis:vis $ident:ident<$model:ty>;

        update_one($model_name:ident) $update_block:block;
        delete_one_by_id($id_name:ident) $delete_block:block;

        $($tokens:tt)*
    } => {
        $crate::repository!(
            $(#[$meta])*
            $vis $ident<$model>;

            #[inline]
            fn insert_one(_model: &$model) -> $crate::types::Query {
                unimplemented!("Insert is not implemented for this repository");
            }

            #[inline]
            fn update_one($model_name: &$model) -> $crate::types::Query $update_block

            #[inline]
            fn delete_one_by_id($id_name: &<$model as Model>::Id) -> $crate::types::Query $delete_block

            $($tokens)*
        );
    };

    {
        $( #[$meta:meta] )*
        $vis:vis $ident:ident<$model:ty>;

        insert_one($insert_model_name:ident) $insert_block:block;
        update_one($update_model_name:ident) $update_block:block;
        delete_one_by_id($id_name:ident) $delete_block:block;

        $($tokens:tt)*
    } => {
        $crate::repository!(
            $(#[$meta])*
            $vis $ident<$model>;

            #[inline]
            fn insert_one($insert_model_name: &$model) -> $crate::types::Query $insert_block

            #[inline]
            fn update_one($update_model_name_name: &$model) -> $crate::types::Query $update_block

            #[inline]
            fn delete_one_by_id($id_name: &<$model as Model>::Id) -> $crate::types::Query $delete_block

            $($tokens)*
        );
    };

    {
        $( #[$meta:meta] )*
        $vis:vis $ident:ident<$model:ty>;

        insert_one($insert_model_name:ident) $insert_block:block;
        update_one($update_model_name:ident) $update_block:block;

        $($tokens:tt)*
    } => {
        $crate::repository!(
            $(#[$meta])*
            $vis $ident<$model>;

            #[inline]
            fn insert_one($insert_model_name: &$model) -> $crate::types::Query $insert_block

            #[inline]
            fn update_one($update_model_name_name: &$model) -> $crate::types::Query $update_block

            #[inline]
            fn delete_one_by_id(_: &<$model as Model>::Id) -> $crate::types::Query {
                unimplemented!("Delete is not implemented for this repository");
            }

            $($tokens)*
        );
    };

    {
        $( #[$meta:meta] )*
        $vis:vis $ident:ident<$model:ty>;

        insert_one($insert_model_name:ident) $insert_block:block;
        delete_one_by_id($id_name:ident) $delete_block:block;

        $($tokens:tt)*
    } => {
        $crate::repository!(
            $(#[$meta])*
            $vis $ident<$model>;

            #[inline]
            fn insert_one($insert_model_name: &$model) -> $crate::types::Query $insert_block

            #[inline]
            fn update_one(_: &$model) -> $crate::types::Query {
                unimplemented!("Update is not implemented for this repository");
            }

            #[inline]
            fn delete_one_by_id($id_name: &<$model as Model>::Id) -> $crate::types::Query $delete_block

            $($tokens)*
        );
    };

    {
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