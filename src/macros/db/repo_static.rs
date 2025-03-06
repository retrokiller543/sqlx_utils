#[macro_export]
macro_rules! static_repo {
    /*(
        $vis:vis $ident:ident;
        $init_method:ident($($param:pat_param = $param_ty:ty),*) -> $return_ty:ty $init_block:block
    ) => {
        ::paste::paste! {
            $vis static [<$ident:snake:upper>]: ::tokio::sync::OnceCell<$return_ty> = ::tokio::sync::OnceCell::const_new();

            #[inline(always)]
            #[tracing::instrument(level = "debug")]
            fn $init_method($($param: $param_ty),*) -> $crate::error::Result<$return_ty> $init_block

            #[inline(always)]
            #[tracing::instrument(level = "debug")]
            $vis fn [<get_ $ident:snake>]() -> $crate::error::Result<&'static $return_ty> {
                [<$ident:snake:upper>].get_or_try_init($init_method).await
            }
        }
    };

    ($vis:vis $ident:ident;) => {
        ::paste::paste! {
            $crate::static_repo!(
                $vis $ident;

                [<init_ $ident:snake:lower>]() -> $ident {
                    $ident::new().await
                }
            );
        }
    };*/

    ($vis:vis $ident:ident;) => {
        ::paste::paste! {
            $vis static [<$ident:snake:upper>]: ::std::sync::LazyLock<$ident> = ::std::sync::LazyLock::new(|| {
                $ident::new()
            });
        }
    };

    (!zst $vis:vis $ident:ident;) => {
        ::paste::paste! {
            $vis static [<$ident:snake:upper>]: $ident = $ident::new();
        }
    };
}
