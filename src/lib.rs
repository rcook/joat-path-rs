#![feature(pattern)]

mod path_clean;

pub use self::path_clean::{clean, clean_unix, clean_windows, PathClean};
