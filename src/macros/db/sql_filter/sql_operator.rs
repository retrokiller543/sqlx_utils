#[doc(hidden)]
#[macro_export]
macro_rules! sql_operator {
    ($vis:vis $ident:ident, $lit:literal) => {
        $vis struct $ident<T> {
            column: &'static str,
            value: Option<T>,
        }

        impl<'args, T> $ident<T>
        where
            T: ::sqlx::Type<$crate::types::Database> + ::sqlx::Encode<'args, $crate::types::Database> + 'args,
        {
            #[inline]
            $vis fn new(column: &'static str, value: Option<T>) -> Self {
                Self { column, value }
            }
        }

        impl $ident<$crate::filter::Raw> {
            #[inline]
            $vis fn new_raw(column: &'static str, value: $crate::filter::Raw) -> Self {
                Self { column, value: Some(value) }
            }
        }

        $crate::sql_impl! {
            $ident<T>;

            apply_filter(s, builder) {
                if let Some(val) = s.value {
                    builder.push(s.column);
                    builder.push(concat!(" ", $lit, " "));
                    builder.push_bind(val);
                }
            }

            should_apply_filter(s) {
                s.value.is_some()
            }

            where
                T: ::sqlx::Type<$crate::types::Database> + ::sqlx::Encode<'args, $crate::types::Database>
        }

        impl<'args> $crate::traits::SqlFilter<'args> for $ident<$crate::filter::Raw>
        {
            #[inline]
            fn apply_filter(self, builder: &mut ::sqlx::QueryBuilder<'args, $crate::types::Database>) {
                if let Some(val) = self.value {
                    builder.push(self.column);
                    builder.push(concat!(" ", $lit, " "));
                    val.apply_filter(builder)
                }

            }

            #[inline]
            fn should_apply_filter(&self) -> bool {
                self.value.is_some()
            }
        }

        ::paste::paste! {
            #[inline]
            pub fn [< $ident:snake >]<'args, T>(
                column: &'static str,
                value: Option<T>
            ) -> $crate::filter::Filter<$ident<T>>
            where
                T: ::sqlx::Type<$crate::types::Database> + ::sqlx::Encode<'args, $crate::types::Database> + 'args,
            {
                $crate::filter::Filter::new($ident::new(column, value))
            }

            #[inline]
            pub fn [< $ident:snake _raw >](
                column: &'static str,
                value: $crate::filter::Raw
            ) -> $crate::filter::Filter<$ident<$crate::filter::Raw>> {
                $crate::filter::Filter::new($ident::new_raw(column, value))
            }
        }
    };

    ($vis:vis $ident:ident<$ty:ty>, $lit:literal) => {
        $vis struct $ident {
            column: &'static str,
            value: Option<$ty>,
        }

        impl $ident {
            #[inline]
            $vis fn new(column: &'static str, value: Option<impl Into<$ty>>) -> Self {
                let value = value.map(::core::convert::Into::into);
                Self { column, value }
            }
        }

        $crate::sql_impl! {
            $ident;

            apply_filter(s, builder) {
                if let Some(value) = s.value {
                    builder.push(s.column);
                    builder.push(concat!(" ", $lit, " "));
                    builder.push_bind(value);
                }
            }

            should_apply_filter(s) {
                s.value.is_some()
            }
        }

        ::paste::paste! {
            #[inline]
            pub fn [< $ident:snake >](
                column: &'static str,
                value: Option<impl Into<$ty>>
            ) -> $crate::filter::Filter<$ident> {
                $crate::filter::Filter::new($ident::new(column, value))
            }

            #[inline]
            pub fn [< $ident:snake _raw >](
                column: &'static str,
                value: $crate::filter::Raw
            ) -> $crate::filter::Filter<$ident> {
                $crate::filter::Filter::new($ident::new(column, Some(value.0)))
            }
        }
    };

    ($vis:vis $ident:ident[], $lit:literal) => {
        $vis struct $ident<T> {
            column: &'static str,
            values: Vec<T>,
        }

        impl<'args, T> $ident<T>
        where
            T: ::sqlx::Type<$crate::types::Database> + ::sqlx::Encode<'args, $crate::types::Database> + 'args,
        {
            #[inline]
            $vis fn new(column: &'static str, values: impl IntoIterator<Item = T>) -> Self {
                let values = values.into_iter().collect();
                Self { column, values }
            }
        }

        $crate::sql_impl! {
            $ident<T>;

            apply_filter(s, builder) {
                if !s.should_apply_filter()  {
                    return;
                }

                builder.push(s.column);
                builder.push(concat!(" ", $lit, " ("));

                let mut first = true;
                for val in s.values {
                    if !first {
                        builder.push(", ");
                    }
                    builder.push_bind(val);
                    first = false;
                }

                builder.push(")");
            }

            should_apply_filter(s) {
                !s.values.is_empty()
            }

            where
                T: ::sqlx::Type<$crate::types::Database> + ::sqlx::Encode<'args, $crate::types::Database>
        }

        ::paste::paste! {
            #[inline]
            pub fn [< $ident:snake >]<'args, T>(
                column: &'static str,
                values: impl IntoIterator<Item = T>
            ) -> $crate::filter::Filter<$ident<T>>
            where
                T: ::sqlx::Type<$crate::types::Database> + ::sqlx::Encode<'args, $crate::types::Database> + 'args,
            {
                $crate::filter::Filter::new($ident::new(column, values))
            }
        }
    };

    ($vis:vis $ident:ident) => {
        $vis struct $ident;

        impl Default for $ident {
            fn default() -> Self {
                Self::new()
            }
        }

        impl $ident {
            #[inline]
            $vis fn new() -> Self {
                Self
            }
        }

        $crate::sql_impl! {
            $ident;
            apply_filter(_, _) {}
            should_apply_filter(_) { false }
        }

        ::paste::paste! {
            #[inline]
            pub fn [< $ident:snake >]() -> $crate::filter::Filter<$ident> {
                $crate::filter::Filter::new($ident::new())
            }

            #[inline]
            pub fn [< $ident:snake _raw >]() -> $crate::filter::Filter<$ident> {
                $crate::filter::Filter::new($ident::new())
            }
        }
    };
}
