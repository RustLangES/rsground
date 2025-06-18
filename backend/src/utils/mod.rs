mod async_from;
mod expect_var;
mod to_stream;

use std::sync::{Arc, LazyLock};

pub use async_from::AsyncInto;
pub use expect_var::expect_var;
pub use to_stream::ToStream;

/// Use `Arc` instead of `String` because **user's id** is a immutable string
/// and is shared across threads.
///
/// https://blocklisted.github.io/blog/arc_str_vs_string_is_it_really_faster/
pub type ArcStr = std::sync::Arc<str>;

pub static EMPTY_STR: LazyLock<ArcStr> = LazyLock::new(|| Arc::from(""));
