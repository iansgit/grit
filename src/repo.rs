use std::path::{Path, PathBuf};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum RepoError {
    #[error("not a git repository (or any parent directory): {0}")]
    NotFound(PathBuf),
    #[error("repository already exists at {0}")]
    AlreadyExists(PathBuf),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// A handle to an open grit repository.
pub struct Repo {
    /// The root of the working tree (the directory containing `.git`).
    pub workdir: PathBuf,
}

impl Repo {
    /// Return the path to the `.git` directory.
    pub fn git_dir(&self) -> PathBuf {
        self.workdir.join(".git")
    }

    /// Walk up from `path` to find the nearest `.git` directory.
    pub fn discover(path: &Path) -> Result<Self, RepoError> {
        let mut current = path.to_path_buf();
        loop {
            if current.join(".git").is_dir() {
                return Ok(Repo { workdir: current });
            }
            if !current.pop() {
                return Err(RepoError::NotFound(path.to_path_buf()));
            }
        }
    }
}
