#[macro_export]
macro_rules! expect_var {
    ($name:literal) => {
        ::std::env::var($name).expect(concat!("set \"", $name, "\" env var"))
    };
}

#[allow(unused_imports)]
pub use expect_var;
