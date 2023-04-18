#![feature(pattern)]

mod absolute_path;
mod path_clean;

pub use self::absolute_path::absolute_path;
pub use self::path_clean::{clean, clean_unix, clean_windows, PathClean};
