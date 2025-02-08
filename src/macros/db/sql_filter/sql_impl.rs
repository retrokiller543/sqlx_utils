#[macro_export]
macro_rules! sql_impl {
    // Base case with explicit lifetime bounds and where clauses
    {
        $ident:ident $(<
            $($lt:lifetime,)*
            $($generic:ident $(: $($generic_bound:tt)+)?),*
            $(,)?
        >)?;
        $apply_filter:ident ($apply_self:ident, $builder:ident) $apply_block:block
        $should_apply_filter:ident($should_self:ident) $should_apply_block:block
        $(where $($where_clause:tt)+)?
    } => {
        impl<'args $(, $($lt,)* $($generic),*)?> $crate::traits::SqlFilter<'args>
            for $ident$(<$($lt,)* $($generic),*>)?
        where
            $($($generic: $($($generic_bound)+ +)? 'args,)*)?
            $($($where_clause)+)?
        {
            #[inline]
            fn $apply_filter(self, builder: &mut ::sqlx::QueryBuilder<'args, $crate::types::Database>) {
                let filter_impl = |$apply_self: Self, $builder: &mut ::sqlx::QueryBuilder<'args, $crate::types::Database>|
                    $apply_block;
                filter_impl(self, builder)
            }

            #[inline]
            fn $should_apply_filter(&self) -> bool {
                let should_apply_impl = |$should_self: &Self| $should_apply_block;
                should_apply_impl(self)
            }
        }
    };

    // Shorthand case for simple SqlFilter bounds
    {
        $ident:ident $(<$($lt:lifetime,)* $($generic:ident),* $(,)?>)?;
        $apply_filter:ident ($apply_self:ident, $builder:ident) $apply_block:block
        $should_apply_filter:ident($should_self:ident) $should_apply_block:block
    } => {
        $crate::traits::sql_impl! {
            $ident$(<$($lt,)* $($generic: $crate::traits::SqlFilter<'args>),*>)?;
            $apply_filter($apply_self, $builder) $apply_block
            $should_apply_filter($should_self) $should_apply_block
            where
        }
    };

    {
        $ident:ident $(<$($lt:lifetime)? $(,)? $($generic:ident),* $(,)?>)?;
        $apply_filter:ident (_, _) $apply_block:block
        $should_apply_filter:ident(_) $should_apply_block:block
    } => {
        impl<'args, $($($lt,)? $($generic),*)?> $crate::traits::SqlFilter<'args> for $ident$(<$($lt,)? $($generic),*>)?
        $(where
            $($generic: $crate::traits::SqlFilter<'args> + 'args),*)?
        {
            #[inline]
            fn $apply_filter(self, builder: &mut ::sqlx::QueryBuilder<'args, $crate::types::Database>) {
                let filter_impl = |_: Self, _: &mut ::sqlx::QueryBuilder<'args, $crate::types::Database>| $apply_block;
                filter_impl(self, builder);
            }

            #[inline]
            fn $should_apply_filter(&self) -> bool {
                let should_apply_impl = |_: &Self| $should_apply_block;
                should_apply_impl(self)
            }
        }
    };
}
