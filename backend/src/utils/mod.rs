mod async_default;
mod async_from;
mod expect_var;
mod logger;
mod to_stream;

pub use async_default::AsyncDefault;
pub use async_from::AsyncInto;
pub use expect_var::expect_var;
pub(crate) use logger::define_local_logger;
pub use to_stream::ToStream;

use std::sync::{Arc, LazyLock};

/// Use `Arc` instead of `String` because **user's id** is a immutable string
/// and is shared across threads.
///
/// https://blocklisted.github.io/blog/arc_str_vs_string_is_it_really_faster/
pub type ArcStr = Arc<str>;

pub static EMPTY_STR: LazyLock<ArcStr> = LazyLock::new(|| Arc::from(""));
