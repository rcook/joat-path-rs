//! `path-clean` is a Rust port of the the `cleanname` procedure from the Plan 9 C library, and is similar to
//! [`path.Clean`](https://golang.org/pkg/path/#Clean) from the Go standard library. It works as follows:
//!
//! 1. Reduce multiple slashes to a single slash.
//! 2. Eliminate `.` path name elements (the current directory).
//! 3. Eliminate `..` path name elements (the parent directory) and the non-`.` non-`..`, element that precedes them.
//! 4. Eliminate `..` elements that begin a rooted path, that is, replace `/..` by `/` at the beginning of a path.
//! 5. Leave intact `..` elements that begin a non-rooted path.
//!
//! If the result of this process is an empty string, return the string `"."`, representing the current directory.
//!
//! It performs this transform lexically, without touching the filesystem. Therefore it doesn't do
//! any symlink resolution or absolute path resolution. For more information you can see ["Getting Dot-Dot
//! Right"](https://9p.io/sys/doc/lexnames.html).
//!
//! For convenience, the [`PathClean`] trait is exposed and comes implemented for [`std::path::PathBuf`].
//!
//! ```rust
//! use std::path::PathBuf;
//! use joat_path::{clean, PathClean};
//! assert_eq!(clean("hello/world/.."), "hello");
//! assert_eq!(
//!     PathBuf::from("/test/../path/").clean(),
//!     PathBuf::from("/path")
//! );
//! ```
use self::internal::PathCharacteristics;
use std::path::PathBuf;

/// The Clean trait implements a `clean` method. It's recommended you use the provided [`clean`]
/// function.
pub trait PathClean<T> {
    fn clean(&self) -> T;
}

/// `PathClean` implemented for `PathBuf`
impl PathClean<Self> for PathBuf {
    fn clean(&self) -> Self {
        Self::from(clean(self.to_str().unwrap_or("")))
    }
}

mod internal {
    pub trait PathCharacteristics {
        /// Primary separator character for this type of path
        const CANONICAL_SEPARATOR: char;

        /// Returns true if string is any valid separator character, false otherwise
        fn is_separator(path: &str) -> bool;

        /// Returns true if string starts with any valid separator character, false otherwise
        fn starts_with_separator(path: &str) -> bool;

        /// Removes separators from end of string
        fn trim_end_matches_separator(path: &str) -> &str;

        /// Split path on separators
        fn split_on_separators(path: &str) -> Vec<&str>;
    }

    /// Characteristics for Unix-style paths
    /// * Path separator is always a single forward slash "/"
    pub struct UnixPath;

    impl PathCharacteristics for UnixPath {
        const CANONICAL_SEPARATOR: char = '/';

        fn is_separator(path: &str) -> bool {
            path == "/"
        }

        fn starts_with_separator(path: &str) -> bool {
            path.starts_with('/')
        }

        fn trim_end_matches_separator(path: &str) -> &str {
            path.trim_end_matches('/')
        }

        fn split_on_separators(path: &str) -> Vec<&str> {
            path.split('/').collect()
        }
    }

    /// Characteristics for Windows-style paths
    /// * Path separator can be a single forward slash "/" or backslash "\\"
    pub struct WindowsPath;

    impl PathCharacteristics for WindowsPath {
        const CANONICAL_SEPARATOR: char = '\\';

        fn is_separator(path: &str) -> bool {
            path == "\\" || path == "/"
        }

        fn starts_with_separator(path: &str) -> bool {
            path.starts_with(['\\', '/'])
        }

        fn trim_end_matches_separator(path: &str) -> &str {
            path.trim_end_matches(['\\', '/'])
        }

        fn split_on_separators(path: &str) -> Vec<&str> {
            path.split(['\\', '/']).collect()
        }
    }

    /// Get normalized version of special path if path is special
    ///
    /// # Arguments
    ///
    /// * `path` - Path
    pub fn special_path<P: PathCharacteristics>(path: &str) -> Option<String> {
        if P::is_separator(path) {
            return Some(P::CANONICAL_SEPARATOR.to_string());
        }
        match path {
            "" | "." => Some(String::from(".")),
            ".." => Some(String::from("..")),
            _ => None,
        }
    }

    /// Determine if path is rooted
    ///
    /// # Arguments
    ///
    /// * `path` - Path
    pub fn is_root<P: PathCharacteristics>(path: &str) -> bool {
        P::starts_with_separator(path)
    }

    /// Trim trailing path separators from end of path
    ///
    /// # Arguments
    ///
    /// * `path` - Path
    pub fn trim_end_path<P: PathCharacteristics>(path: &str) -> &str {
        P::trim_end_matches_separator(path)
    }

    /// Split path into segments based on path characteristics
    ///
    /// # Arguments
    ///
    /// * `path` - Path
    pub fn split_path_segments<P: PathCharacteristics>(path: &str) -> Vec<&str> {
        P::split_on_separators(path)
    }

    /// Join path segments to create a path based on path characteristics
    ///
    /// # Arguments
    ///
    /// * `segments` - Segments
    pub fn join_path_segments<P: PathCharacteristics>(segments: &[&str]) -> String {
        segments.join(&P::CANONICAL_SEPARATOR.to_string())
    }

    /// Make an absolute path based on path characteristics
    ///
    /// # Arguments
    ///
    /// * `path` - Path
    pub fn make_absolute<P: PathCharacteristics>(path: &str) -> String {
        P::CANONICAL_SEPARATOR.to_string() + path
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_special_path_unix() {
            assert_eq!(Some(String::from(".")), special_path::<UnixPath>(""));
            assert_eq!(Some(String::from(".")), special_path::<UnixPath>("."));
            assert_eq!(Some(String::from("..")), special_path::<UnixPath>(".."));
            assert_eq!(Some(String::from("/")), special_path::<UnixPath>("/"));
            assert_eq!(None, special_path::<UnixPath>("\\"));
            assert_eq!(None, special_path::<UnixPath>("aaa"));
        }

        #[test]
        fn test_special_path_windows() {
            assert_eq!(Some(String::from(".")), special_path::<WindowsPath>(""));
            assert_eq!(Some(String::from(".")), special_path::<WindowsPath>("."));
            assert_eq!(Some(String::from("..")), special_path::<WindowsPath>(".."));
            assert_eq!(Some(String::from("\\")), special_path::<WindowsPath>("/"));
            assert_eq!(Some(String::from("\\")), special_path::<WindowsPath>("\\"));
            assert_eq!(None, special_path::<WindowsPath>("aaa"));
        }

        #[test]
        fn test_is_root_unix() {
            assert!(is_root::<UnixPath>("/a"));
            assert!(!is_root::<UnixPath>("\\a"));
            assert!(!is_root::<UnixPath>("a"));
        }

        #[test]
        fn test_is_root_windows() {
            assert!(is_root::<WindowsPath>("/a"));
            assert!(is_root::<WindowsPath>("\\a"));
            assert!(!is_root::<WindowsPath>("a"));
        }

        #[test]
        fn test_trim_end_path_unix() {
            assert_eq!("aaa", trim_end_path::<UnixPath>("aaa"));
            assert_eq!("aaa", trim_end_path::<UnixPath>("aaa/"));
            assert_eq!("aaa", trim_end_path::<UnixPath>("aaa/////"));
        }

        #[test]
        fn test_trim_end_path_windows() {
            assert_eq!("aaa", trim_end_path::<WindowsPath>("aaa"));
            assert_eq!("aaa", trim_end_path::<WindowsPath>("aaa/"));
            assert_eq!("aaa", trim_end_path::<WindowsPath>("aaa\\"));
            assert_eq!("aaa", trim_end_path::<WindowsPath>("aaa/////"));
            assert_eq!("aaa", trim_end_path::<WindowsPath>("aaa\\\\\\\\\\"));
            assert_eq!("aaa", trim_end_path::<WindowsPath>("aaa/\\/\\/"));
        }

        #[test]
        fn test_split_path_segments_first_empty_unix() {
            let segments = split_path_segments::<UnixPath>("/a/b/c");
            assert_eq!(4, segments.len());
            assert_eq!("", segments[0]);
            assert_eq!("a", segments[1]);
            assert_eq!("b", segments[2]);
            assert_eq!("c", segments[3]);
        }

        #[test]
        fn test_split_path_segments_last_empty_unix() {
            let segments = split_path_segments::<UnixPath>("/a/b/c/");
            assert_eq!(5, segments.len());
            assert_eq!("", segments[0]);
            assert_eq!("a", segments[1]);
            assert_eq!("b", segments[2]);
            assert_eq!("c", segments[3]);
            assert_eq!("", segments[4]);
        }

        #[test]
        fn test_split_path_segments_empty_unix() {
            let segments = split_path_segments::<UnixPath>("");
            assert_eq!(1, segments.len());
            assert_eq!("", segments[0]);
        }

        #[test]
        fn test_split_path_segments_multiple_empty_unix() {
            let segments = split_path_segments::<UnixPath>("//");
            assert_eq!(3, segments.len());
            assert_eq!("", segments[0]);
            assert_eq!("", segments[1]);
            assert_eq!("", segments[2]);
        }

        #[test]
        fn test_split_path_segments_first_empty_unix_backslashes() {
            let segments = split_path_segments::<UnixPath>("/a\\b\\c");
            assert_eq!(2, segments.len());
            assert_eq!("", segments[0]);
            assert_eq!("a\\b\\c", segments[1]);
        }

        #[test]
        fn test_split_path_segments_first_empty_windows_backslashes() {
            let segments = split_path_segments::<WindowsPath>("\\a\\b\\c");
            assert_eq!(4, segments.len());
            assert_eq!("", segments[0]);
            assert_eq!("a", segments[1]);
            assert_eq!("b", segments[2]);
            assert_eq!("c", segments[3]);
        }

        #[test]
        fn test_split_path_segments_first_empty_windows_mixture() {
            let segments = split_path_segments::<WindowsPath>("/a\\b/c");
            assert_eq!(4, segments.len());
            assert_eq!("", segments[0]);
            assert_eq!("a", segments[1]);
            assert_eq!("b", segments[2]);
            assert_eq!("c", segments[3]);
        }

        #[test]
        fn test_join_path_segments_unix() {
            assert_eq!("", join_path_segments::<UnixPath>(&[]));
            assert_eq!("a/b/c", join_path_segments::<UnixPath>(&["a", "b", "c"]));
        }

        #[test]
        fn test_join_path_segments_windows() {
            assert_eq!("", join_path_segments::<WindowsPath>(&[]));
            assert_eq!(
                "a\\b\\c",
                join_path_segments::<WindowsPath>(&["a", "b", "c"])
            );
        }

        #[test]
        fn test_make_absolute_unix() {
            assert_eq!("/aaa", make_absolute::<UnixPath>("aaa"));
            assert_eq!("//aaa", make_absolute::<UnixPath>("/aaa"));
            assert_eq!("/\\aaa", make_absolute::<UnixPath>("\\aaa"));
        }

        #[test]
        fn test_make_absolute_windows() {
            assert_eq!("\\aaa", make_absolute::<WindowsPath>("aaa"));
            assert_eq!("\\/aaa", make_absolute::<WindowsPath>("/aaa"));
            assert_eq!("\\\\aaa", make_absolute::<WindowsPath>("\\aaa"));
        }
    }
}

/// The core implementation. It performs the following, lexically:
/// 1. Reduce multiple slashes to a single slash.
/// 2. Eliminate `.` path name elements (the current directory).
/// 3. Eliminate `..` path name elements (the parent directory) and the non-`.` non-`..`, element that precedes them.
/// 4. Eliminate `..` elements that begin a rooted path, that is, replace `/..` by `/` at the beginning of a path.
/// 5. Leave intact `..` elements that begin a non-rooted path.
///
/// If the result of this process is an empty string, return the string `"."`, representing the current directory.
#[must_use]
pub fn clean(path: &str) -> String {
    #[cfg(not(target_os = "windows"))]
    type PlatformPath = internal::UnixPath;
    #[cfg(target_os = "windows")]
    type PlatformPath = internal::WindowsPath;

    clean_core::<PlatformPath>(path)
}

#[must_use]
pub fn clean_unix(path: &str) -> String {
    clean_core::<internal::UnixPath>(path)
}

#[must_use]
pub fn clean_windows(path: &str) -> String {
    clean_core::<internal::WindowsPath>(path)
}

#[allow(clippy::unnecessary_unwrap)]
fn clean_core<P: PathCharacteristics>(path: &str) -> String {
    use internal::{
        is_root, join_path_segments, make_absolute, special_path, split_path_segments,
        trim_end_path,
    };

    if let Some(s) = special_path::<P>(path) {
        return s;
    }

    let mut out = vec![];
    let is_root = is_root::<P>(path);

    let path = trim_end_path::<P>(path);
    let segments = split_path_segments::<P>(path);
    let num_segments = segments.len();

    for segment in segments {
        match segment {
            "" => continue,
            "." => {
                if num_segments == 1 {
                    out.push(segment);
                };
                continue;
            }
            ".." => {
                let previous = out.pop();
                if previous.is_some() && !can_backtrack(previous.unwrap()) {
                    out.push(previous.unwrap());
                    out.push(segment);
                } else if previous.is_none() && !is_root {
                    out.push(segment);
                };
                continue;
            }
            _ => {
                out.push(segment);
            }
        };
    }

    let out_str_0 = join_path_segments::<P>(&out);

    let out_str_1 = if is_root {
        make_absolute::<P>(&out_str_0)
    } else {
        out_str_0
    };

    if out_str_1.is_empty() {
        ".".to_string()
    } else {
        out_str_1
    }
}

fn can_backtrack(segment: &str) -> bool {
    !matches!(segment, "." | "..")
}

#[cfg(test)]
mod tests {
    use super::test_helpers::to_windows;
    use super::{clean_unix, clean_windows, PathClean};

    use std::path::PathBuf;

    #[test]
    fn test_empty_path_is_current_dir() {
        assert_eq!(clean_unix(""), ".");
        assert_eq!(clean_windows(&to_windows("")), to_windows("."));
    }

    #[test]
    fn test_clean_paths_dont_change() {
        let tests = vec![(".", "."), ("..", ".."), ("/", "/")];

        for test in tests {
            assert_eq!(clean_unix(test.0), test.1);
            assert_eq!(clean_windows(&to_windows(test.0)), to_windows(test.1));
        }
    }

    #[test]
    fn test_replace_multiple_slashes() {
        let tests = vec![
            ("/", "/"),
            ("//", "/"),
            ("///", "/"),
            (".//", "."),
            ("//..", "/"),
            ("..//", ".."),
            ("/..//", "/"),
            ("/.//./", "/"),
            ("././/./", "."),
            ("path//to///thing", "path/to/thing"),
            ("/path//to///thing", "/path/to/thing"),
        ];

        for test in tests {
            assert_eq!(clean_unix(test.0), test.1);
            assert_eq!(clean_windows(&to_windows(test.0)), to_windows(test.1));
        }
    }

    #[test]
    fn test_eliminate_current_dir() {
        let tests = vec![
            ("./", "."),
            ("/./", "/"),
            ("./test", "test"),
            ("./test/./path", "test/path"),
            ("/test/./path/", "/test/path"),
            ("test/path/.", "test/path"),
        ];

        for test in tests {
            assert_eq!(clean_unix(test.0), test.1);
            assert_eq!(clean_windows(&to_windows(test.0)), to_windows(test.1));
        }
    }

    #[test]
    fn test_eliminate_parent_dir() {
        let tests = vec![
            ("/..", "/"),
            ("/../test", "/test"),
            ("test/..", "."),
            ("test/path/..", "test"),
            ("test/../path", "path"),
            ("/test/../path", "/path"),
            ("test/path/../../", "."),
            ("test/path/../../..", ".."),
            ("/test/path/../../..", "/"),
            ("/test/path/../../../..", "/"),
            ("test/path/../../../..", "../.."),
            ("test/path/../../another/path", "another/path"),
            ("test/path/../../another/path/..", "another"),
            ("../test", "../test"),
            ("../test/", "../test"),
            ("../test/path", "../test/path"),
            ("../test/..", ".."),
        ];

        for test in tests {
            assert_eq!(clean_unix(test.0), test.1);
            assert_eq!(clean_windows(&to_windows(test.0)), to_windows(test.1));
        }
    }

    #[test]
    fn test_pathbuf_trait() {
        assert_eq!(
            PathBuf::from("/test/../path/").clean(),
            PathBuf::from("/path")
        );
    }
}

#[cfg(test)]
mod test_helpers {
    pub fn to_windows(p: &str) -> String {
        p.replace('/', "\\")
    }
}
