#[macro_export]
macro_rules! repository_update {
    {
        $ident:ident<$model:ty>;
    } => {
        $crate::repository_update!(
            !inner
            $ident<$model>;
        );
    };

    {
        $ident:ident<$model:ty>;

        $method_name:ident($param:pat_param) $block:block
    } => {
        $crate::repository_update!(!inner $ident<$model>; fn $method_name($param: &$model) -> $crate::types::Query<'_> $block);
    };

    {
        $ident:ident<$model:ty>;

        $($tokens:tt)*
    } => {
        $crate::repository_update!(!inner $ident<$model>; $($tokens)*);
    };

    {
        !inner
        $ident:ident<$model:ty>;

        $($tokens:tt)*
    } => {
        impl $crate::traits::UpdatableRepository<$model> for $ident {
            $($tokens)*
        }
    }
}
