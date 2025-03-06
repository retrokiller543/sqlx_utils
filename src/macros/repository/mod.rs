mod delete;
mod update;

#[macro_export]
macro_rules! repository_insert {
    {
        $ident:ident<$model:ty>;
    } => {
        $crate::repository_insert!(
            !inner
            $ident<$model>;
        );
    };

    {
        $ident:ident<$model:ty>;

        fn $method_name:ident($param:pat_param) $block:block;
    } => {
        $crate::repository_insert!(!inner $ident<$model>; fn $method_name($param: &$model) -> $crate::types::Query<'_> $block);
    };

    {
        $ident:ident<$model:ty>;

        $($tokens:tt)*
    } => {
        $crate::repository_insert!(!inner $ident<$model>; $($tokens)*);
    };

    {
        !inner
        $ident:ident<$model:ty>;

        $($tokens:tt)*
    } => {
        impl $crate::traits::InsertableRepository<$model> for $ident {
            $($tokens)*
        }
    }
}
