#[macro_export]
macro_rules! mod_def {
    {$($vis:vis mod $ident:ident $(;)?)+} => {
        $($vis mod $ident;
        #[allow(unused_imports)]
        $vis use $ident::*;)+
    };

    {!export $($vis:vis mod $ident:ident $(;)?)+} => {
        $($vis mod $ident;
        #[allow(unused_imports)]
        pub use $ident::*;)+
    };
}

mod_def! {
    pub mod batch;
}

macro_rules! tracing_debug_log {
    {[$(skip($($ident:ident),*) $(,)? )? $($parent:expr,)? $($name:literal,)?] $($tt:tt)*} => {
        #[cfg_attr(feature = "log_err", tracing::instrument($(skip($($ident),*),)?level = "debug", $(parent = &$parent,)? $(name = $name,)? err))]
        #[cfg_attr(not(feature = "log_err"), tracing::instrument($(skip($($ident),*),)?level = "debug", $(parent = &$parent,)? $(name = $name,)?))]
        $($tt)*
    };

    {[skip_all, $($parent:expr,)? $($name:literal,)?] $($tt:tt)*} => {
        #[cfg_attr(feature = "log_err", tracing::instrument(skip_all, level = "debug", $(parent = &$parent,)? $(name = $name,)? err))]
        #[cfg_attr(not(feature = "log_err"), tracing::instrument(skip_all, level = "debug", $(parent = &$parent,)? $(name = $name,)?))]
        $($tt)*
    };
}
pub(crate) use tracing_debug_log;
