#[macro_export]
macro_rules! sql_delimiter {
    {
        $(#[$struct_meta:meta])*
        $vis:vis struct $ident:ident $(<
            $($lt:lifetime,)*
            $($generic:ident $(: $($generic_bound:tt)+)?),*
            $(,)?
        >)? {
            $(
                $(#[$field_meta:meta])*
                $field_vis:vis $field_ident:ident: $field_ty:ty
            ),* $(,)?
        }

        $apply_filter:ident ($apply_self:ident, $builder:ident) $apply_block:block
        $should_apply_filter:ident($should_self:ident) $should_apply_block:block
        $(where $($where_clause:tt)+)?
    } => {
        $(#[$struct_meta])*
        $vis struct $ident$(<$($lt,)* $($generic),*>)? {
            $(
                $(#[$field_meta])*
                $field_vis $field_ident: $field_ty
            ),*
        }

        impl$(<$($lt,)* $($generic),*>)? $ident$(<$($lt,)* $($generic),*>)?
        $(where $($where_clause)+)?
        {
            #[inline]
            $vis fn new($($field_ident: $field_ty),*) -> Self {
                Self { $($field_ident),* }
            }
        }

        $crate::sql_impl! {
            $ident$(<$($lt,)* $($generic),*>)?;
            $apply_filter($apply_self, $builder) $apply_block
            $should_apply_filter($should_self) $should_apply_block
            $(where
                $($generic: $crate::traits::SqlFilter<'args>),*)?
        }
    };
}