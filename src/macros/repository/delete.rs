#[macro_export]
macro_rules! repository_delete {
    {
        $ident:ident<$model:ty>;
    } => {
        $crate::repository_delete!(
            !inner
            $ident<$model>;
        );
    };

    {
        $ident:ident<$model:ty>;

        $method_name:ident($param:pat_param) $block:block
        $method_name2:ident($param2:pat_param) $block2:block

        $($tokens:tt)*
    } => {
        $crate::repository_delete!(!inner $ident<$model>; fn $method_name($param: &<$model as $crate::traits::Model>::Id) -> $crate::types::Query<'_> $block fn $method_name2<'args>($param2: impl $crate::prelude::SqlFilter<'args>) -> ::sqlx::QueryBuilder<'args, $crate::prelude::Database> $block2 $($tokens)*) ;
    };

    {
        $ident:ident<$model:ty>;

        $($tokens:tt)*
    } => {
        $crate::repository_delete!(!inner $ident<$model>; $($tokens)*);
    };

    {
        !inner
        $ident:ident<$model:ty>;

        $($tokens:tt)*
    } => {
        impl $crate::traits::DeleteRepository<$model> for $ident {
            $($tokens)*
        }
    }
}
