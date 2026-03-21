use std::path::Path;

use anyhow::{Context, Result};

use crate::repo::{Repo, RepoError};

/// Initialize a new repository at `path`.
pub fn run(path: &Path) -> Result<()> {
    let git_dir = path.join(".git");

    if git_dir.exists() {
        return Err(RepoError::AlreadyExists(path.to_path_buf()).into());
    }

    // Create the directory structure that git init produces.
    for dir in &[
        git_dir.join("objects/info"),
        git_dir.join("objects/pack"),
        git_dir.join("refs/heads"),
        git_dir.join("refs/tags"),
    ] {
        std::fs::create_dir_all(dir)
            .with_context(|| format!("failed to create directory {}", dir.display()))?;
    }

    // Write HEAD pointing to the default branch.
    std::fs::write(git_dir.join("HEAD"), "ref: refs/heads/main\n")
        .context("failed to write HEAD")?;

    // Write a minimal config.
    std::fs::write(
        git_dir.join("config"),
        "[core]\n\trepositoryformatversion = 0\n\tfilemode = true\n\tbare = false\n",
    )
    .context("failed to write config")?;

    let _ = Repo { workdir: path.to_path_buf() }; // validate the structure we just created
    println!("Initialized empty Grit repository in {}", git_dir.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn init_creates_git_structure() {
        let tmp = TempDir::new().unwrap();
        run(tmp.path()).unwrap();

        let git_dir = tmp.path().join(".git");
        assert!(git_dir.is_dir());
        assert!(git_dir.join("HEAD").is_file());
        assert!(git_dir.join("config").is_file());
        assert!(git_dir.join("objects/info").is_dir());
        assert!(git_dir.join("objects/pack").is_dir());
        assert!(git_dir.join("refs/heads").is_dir());
        assert!(git_dir.join("refs/tags").is_dir());
    }

    #[test]
    fn init_head_points_to_main() {
        let tmp = TempDir::new().unwrap();
        run(tmp.path()).unwrap();

        let head = std::fs::read_to_string(tmp.path().join(".git/HEAD")).unwrap();
        assert_eq!(head, "ref: refs/heads/main\n");
    }

    #[test]
    fn init_fails_if_already_initialized() {
        let tmp = TempDir::new().unwrap();
        run(tmp.path()).unwrap();
        assert!(run(tmp.path()).is_err());
    }
}
