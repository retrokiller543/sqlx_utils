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
    } => {
        $crate::repository_delete!(!inner $ident<$model>; fn $method_name($param: &<$model as $crate::traits::Model>::Id) -> $crate::types::Query<'_> $block);
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
