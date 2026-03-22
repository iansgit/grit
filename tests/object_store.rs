mod common;

use grit::objects::{Object, ObjectKind, Store};

/// Write a blob with grit, then verify git can read it back correctly.
#[test]
fn grit_write_git_read() {
    let (dir, repo) = common::grit_repo();
    let store = Store::new(&repo);

    let content = b"hello from grit\n";
    let obj = Object::new(ObjectKind::Blob, content.to_vec());
    let oid = store.write(&obj).unwrap();

    let git_content = common::git_read_blob(dir.path(), &oid.to_string());
    assert_eq!(git_content, content);
}

/// Write a blob with git, then verify grit can read it back correctly.
#[test]
fn git_write_grit_read() {
    let (dir, repo) = common::grit_repo();
    let store = Store::new(&repo);

    let content = b"hello from git\n";
    let oid_str = common::git_write_object(dir.path(), content);

    let oid = oid_str.parse().unwrap();
    let obj = store.read(&oid).unwrap();
    assert_eq!(obj.kind, ObjectKind::Blob);
    assert_eq!(obj.data, content);
}
