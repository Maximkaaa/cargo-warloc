//! Module defining various filepath-related utilities.

use std::path::{Path, PathBuf};

/// Diffs the two paths and returns a (common_path, path1_extra, path2_extra) triplet.
///
/// It does *not* account for symlinks or `..`, use [std::fs::canonicalize()] beforehand.
///
/// Follows the path element splitting rules of [Path::components()], most notably:
///     * it ignores trailing or doubles slashes (e.g. `"/a//b/" == "/a/b"`)
///     * it ignores CWD `.` in the middle of paths (e.g. `"a/./b" == "a/b"`)
pub fn diff_paths(path1: impl AsRef<Path>, path2: impl AsRef<Path>) -> (PathBuf, PathBuf, PathBuf) {
    let (mut common, mut extra_path1, mut extra_path2) =
        (PathBuf::new(), PathBuf::new(), PathBuf::new());

    let mut path_1_components = path1.as_ref().components();
    let mut path_2_components = path2.as_ref().components();

    for component1 in path_1_components.by_ref() {
        let component2 = path_2_components.next();

        match (component1, component2) {
            (c1, Some(c2)) if c1 == c2 => common.push(c1),
            (c1, None) => {
                extra_path1.push(c1);
                break;
            }
            (c1, Some(c2)) => {
                extra_path1.push(c1);
                extra_path2.push(c2);
                break;
            }
        }
    }

    // Drain remainders and return:
    for c1 in path_1_components {
        extra_path1.push(c1)
    }
    for c2 in path_2_components {
        extra_path2.push(c2)
    }

    (common, extra_path1, extra_path2)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::diff_paths;

    fn run_test(path1: &str, path2: &str, result: (&str, &str, &str)) {
        assert_eq!(
            diff_paths(path1, path2),
            (
                PathBuf::from(result.0),
                PathBuf::from(result.1),
                PathBuf::from(result.2)
            )
        )
    }

    #[test]
    fn diff_empty() {
        run_test("", "", ("", "", ""));
        run_test("a", "", ("", "a", ""));
        run_test("", "a", ("", "", "a"));
    }

    #[test]
    fn diff_absolute() {
        run_test("/a", "/b", ("/", "a", "b"));
        run_test("/a/b", "/a/c", ("/a", "b", "c"));
        run_test("/a/b/c", "/a/c", ("/a", "b/c", "c"));
        run_test("/a/b", "/a/c/b", ("/a", "b", "c/b"));
        run_test("/a/b/c", "/a/b/c", ("/a/b/c", "", ""));
    }

    #[test]
    fn diff_relative() {
        run_test("a", "a", ("a", "", ""));
        run_test("a", "b", ("", "a", "b"));
        run_test("a/b", "a", ("a", "b", ""));
        run_test("a/c", "c/b", ("", "a/c", "c/b"));
        run_test("a/b/c", "a/b/c", ("a/b/c", "", ""));
    }

    #[test]
    fn diff_abolute_and_relative() {
        run_test("/a", "a", ("", "/a", "a"));
        run_test("a", "/a", ("", "a", "/a"));
        run_test("/a/b", "a/b", ("", "/a/b", "a/b"));
    }

    #[test]
    fn cwd() {
        run_test("./", ".", (".", "", ""));
        run_test(".", "././", (".", "", ""));
        run_test("././", ".", (".", "", ""));
        // NOTE: Path{Buf}::components() skips `.` in middle of paths:
        run_test("a/./b", "a/b", ("a/b", "", ""));
        run_test("./a/./b", "a/./b", ("", "./a/b", "a/b"));
    }

    #[test]
    fn diff_parent_dir() {
        run_test("a/..", "a", ("a", "..", ""));
        run_test("a/b/../", "a/b", ("a/b", "..", ""));
    }

    #[test]
    fn diff_slashes() {
        run_test("/", "/", ("/", "", ""));
        run_test("///", "//", ("/", "", ""));
        // NOTE: Path{Buf}::components() ignores duplicated/trailing slashes:
        run_test("/a/b", "///a//b///", ("/a/b", "", ""));
    }
}
