# grit

A reimplementation of git in Rust, built as a learning exercise.

**This is not a production tool.** The goal is to understand git deeply by building it from scratch, and to grow familiarity with Rust along the way.

## What this is

grit is a ground-up reimplementation of git, written to be compatible with real git repositories. A repo created by grit should work with git, and a repo created by git should work with grit. This compatibility requirement is intentional — it forces precision, since real git will reject anything that doesn't conform to the spec.

The implementation follows the [git documentation and file format specs](https://git-scm.com/docs) rather than reverse-engineering behavior from the git binary.

## Topics covered

- **Object model** — how git stores blobs, trees, commits, and tags as content-addressed objects on disk
- **Index / staging area** — the binary format of `.git/index` and how git tracks what's staged for commit
- **Refs, branches, and HEAD** — how git represents the current state and history of a repository
- **Packfile format** — how git efficiently stores and transfers object history using delta compression
- **Transfer protocol** — the git wire protocol (`upload-pack`, `receive-pack`) used for clone, fetch, and push
- **Filesystem interaction** — how git interacts with the OS: stat caching, atomic file writes, and file locking

## Compatibility

grit aims to be a drop-in complement to git:

- Repositories initialized by `grit init` are valid git repositories
- Objects written by grit can be read by git, and vice versa
- Wire protocol support means grit can push to and pull from standard git remotes

## Disclaimer

This project is a learning exercise. It is incomplete, likely has bugs, and should not be used for anything important.
