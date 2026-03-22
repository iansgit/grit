use std::path::Path;
use std::process::Command;

use tempfile::TempDir;

/// Create a temp directory initialised as a grit repo.
pub fn grit_repo() -> (TempDir, grit::repo::Repo) {
    let dir = TempDir::new().unwrap();
    grit::commands::init::run(dir.path()).unwrap();
    let repo = grit::repo::Repo::discover(dir.path()).unwrap();
    (dir, repo)
}

/// Run `git hash-object -w --stdin` in `dir`, feeding `content` on stdin.
/// Returns the 40-char hex OID that git prints.
pub fn git_write_object(dir: &Path, content: &[u8]) -> String {
    let mut child = Command::new("git")
        .args(["hash-object", "-w", "--stdin"])
        .current_dir(dir)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();

    use std::io::Write;
    child.stdin.take().unwrap().write_all(content).unwrap();

    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    String::from_utf8(output.stdout).unwrap().trim().to_string()
}

/// Run `git cat-file blob <oid>` in `dir`. Returns the raw content bytes.
pub fn git_read_blob(dir: &Path, oid: &str) -> Vec<u8> {
    let output = Command::new("git")
        .args(["cat-file", "blob", oid])
        .current_dir(dir)
        .output()
        .unwrap();
    assert!(output.status.success());
    output.stdout
}
