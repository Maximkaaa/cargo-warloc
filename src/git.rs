//! Module implementing simple API for fetching LoC info using `git2-rs`

use std::{fmt::Debug, path::Path};

use git2::{Blame, BlameOptions, Error as GitError, Repository};

use crate::pathutil::diff_paths;

/// Simple wrapper over a u64 holding milliseconds seconds since UNIX Epoch.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SecondsSinceEpoch(pub i128);

/// Git context containing a handle on the current git repository
/// and offering a mid-level API for git blame operations.
pub struct GitContext {
    /// Handle on the `git2::Repository`.
    pub repo: Repository,
}

impl GitContext {
    /// Loads the [GitContext] for the repository at or above the given path.
    pub fn from_path_in_repo(path: impl AsRef<Path>) -> Result<Self, GitError> {
        let repo = Repository::discover(&path)?;
        Ok(GitContext { repo })
    }

    /// Loads the [GitBlameContext] for the file at the given path.
    pub fn blame_file(&self, filepath: impl AsRef<Path>) -> Result<GitBlameContext<'_>, GitError> {
        let (_common, repo_extra, file_relative) = diff_paths(
            self.repo
                .path()
                .parent()
                .expect("Git repo should always have a parent dir"),
            &filepath,
        );

        if repo_extra.parent().is_none() {
            GitBlameContext::from_filepath(self, &file_relative)
        } else {
            // File is outside of git repo, let the error propagate:
            GitBlameContext::from_filepath(self, filepath.as_ref())
        }
    }
}

/// Context for a single file's blame data.
pub struct GitBlameContext<'a> {
    git_context: &'a GitContext,
    blame: Blame<'a>,
}

impl<'a> GitBlameContext<'a> {
    /// Loads blame data for the provided file.
    fn from_filepath(
        git_context: &'a GitContext,
        filepath: &Path,
    ) -> Result<GitBlameContext<'a>, GitError> {
        let mut blame_options = BlameOptions::new();
        blame_options
            .track_copies_same_commit_moves(true)
            .track_copies_same_commit_copies(true)
            .first_parent(true);

        let blame = git_context
            .repo
            .blame_file(filepath, Some(&mut blame_options))?;

        Ok(GitBlameContext { git_context, blame })
    }

    /// Returns the [SecondsSinceEpoch] for the line of code with the given number (if it exists)
    pub fn timestamp_for_line(
        &self,
        line_number: usize,
    ) -> Result<Option<SecondsSinceEpoch>, GitError> {
        let hunk = match self.blame.get_line(line_number) {
            None => return Ok(None),
            Some(h) => h,
        };

        let final_commit = self.git_context.repo.find_commit(hunk.final_commit_id())?;

        let timestamp = SecondsSinceEpoch(i128::from(final_commit.time().seconds()));

        Ok(Some(timestamp))
    }
}

impl Debug for GitContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GitContext")
            .field("repo", &self.repo.path())
            .finish()
    }
}

impl<'a> Debug for GitBlameContext<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GitBlameContext")
            .field("git_context", &self.git_context)
            .finish()
    }
}
