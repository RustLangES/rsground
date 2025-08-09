macro_rules! define_local_logger {
    (@macro
        [$D:tt] // dollar sign to emit
        [$mod:ident]
        [$target:expr]
        [$kind:ident]
    ) => {
        mod $mod {
            #[allow(unused_macros)]
            macro_rules! log_macro {
                ($D ($D tt:tt)+) => {::log::$kind!(target: $target, $D ($D tt)+)};
                () => {};
            }

            pub(crate) use log_macro;
        }

        #[allow(unused_imports)]
        pub(crate) use $mod::log_macro as $kind;

    };
    (@mod
        [$D:tt] // dollar sign to emit
        [$mod:ident]
        [$target:expr]
    ) => {
        $crate::utils::define_local_logger!{@macro [$D] [debug_mod] [$target] [debug]}
        $crate::utils::define_local_logger!{@macro [$D] [error_mod] [$target] [error]}
        $crate::utils::define_local_logger!{@macro [$D] [info_mod] [$target] [info]}
        $crate::utils::define_local_logger!{@macro [$D] [trace_mod] [$target] [trace]}
        $crate::utils::define_local_logger!{@macro [$D] [warn_mod] [$target] [warn]}
    };

    (@inner
        [$D:tt] // dollar sign to emit
        [$vis:vis]
        [$prefix:literal]
        [$mod:ident]
        [$([$name:ident] [$($suffix:literal)?]),* ]
    ) => {
        $vis mod $mod {
            $crate::utils::define_local_logger!{ @mod [$D] [$mod] [$prefix] }

            $(
                pub mod $name {
                    $crate::utils::define_local_logger!{ @mod [$D] [$name] [define_local_logger!(@suffix [$prefix] $($suffix)? or $name)] }
                }
            )*
        }
    };

    (@suffix [$prefix:literal] or $name:ident) => {::std::concat!($prefix, "::", ::std::stringify!($name))};
    (@suffix [$prefix:literal] $suffix:literal or $name:ident) => {::std::concat!($prefix, "::", $suffix)};


    ($vis:vis $mod:ident as $prefix:literal {
        $($name:ident $(=> $suffix:literal)?),*
        $(,)?
    }) => { $crate::utils::define_local_logger!{@inner
        [$]
        [$vis]
        [$prefix]
        [$mod]
        [$([$name] [$($suffix)?]),*]
    } }
}

pub(crate) use define_local_logger;
