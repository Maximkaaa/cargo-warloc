//! Module implementing simple API for fetching LoC info using `git2-rs`

use std::{cell::RefCell, collections::HashMap, fmt::Debug, ops::Deref, path::Path};

use chrono::DateTime;
use git2::{Blame, BlameHunk, BlameOptions, Error as GitError, Oid, Repository};

/// Simple wrapper over a u64 holding milliseconds seconds since UNIX Epoch.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SecondsSinceEpoch(i128);

/// Git context containing a handle on the current git repository
/// and offering a mid-level API for git blame operations.
pub struct GitContext {
    /// Handle on the `git2::Repository`.
    repo: Repository,

    /// Simple commit timestamp cache to save on commit object lookups.
    // FIXME: this has the potential of growing to huge sizes for huge repos.
    commit_timestamp_cache: RefCell<HashMap<Oid, SecondsSinceEpoch>>,
}

impl GitContext {
    /// Loads the [GitContext] for the repository at or above the given path.
    pub fn from_repo_root(path: impl AsRef<Path>) -> Result<Self, GitError> {
        Ok(GitContext {
            repo: Repository::discover(path)?,
            commit_timestamp_cache: RefCell::new(HashMap::new()),
        })
    }

    /// Loads the [GitBlameContext] for the file at the given path.
    pub fn blame_file(
        &mut self,
        filepath: impl AsRef<Path>,
    ) -> Result<GitBlameContext<'_>, GitError> {
        GitBlameContext::from_filepath(self, filepath.as_ref())
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
        let blame = git_context
            .repo
            .blame_file(filepath, Some(&mut BlameOptions::default()))?;
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

        let commit_id = hunk.final_commit_id();
        let mut cache = self.git_context.commit_timestamp_cache.borrow_mut();

        let timestamp = match cache.get(&commit_id) {
            Some(ts) => ts.clone(),
            None => {
                let commit = self.git_context.repo.find_commit(hunk.final_commit_id())?;
                let timestamp = SecondsSinceEpoch(i128::from(commit.time().seconds()));
                cache.insert(commit_id, timestamp);
                timestamp
            }
        };

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
