# grit commands

A reference for building grit and manually verifying its behaviour against real git.

## Building

```bash
cargo build
```

The binary is at `target/debug/grit`. For convenience during manual testing, you can install it into your PATH:

```bash
cargo install --path .
```

Or just invoke it directly as `./target/debug/grit` from the repo root.

## Commands

### `grit init`

Initialises a new grit repository.

```bash
grit init                  # initialise in the current directory
grit init path/to/dir      # initialise in a specific directory
```

**Verifying against git:**

A repo initialised by grit should be fully readable by git. To check:

```bash
grit init myrepo
cd myrepo
git status                 # should report "On branch main, no commits yet"
git log                    # should report "fatal: your current branch 'main' does not have any commits yet"
```

### Object store

The object store is currently only accessible as a library — no porcelain commands yet. The following shows how to exercise it manually using git's plumbing commands against a grit-initialised repo.

**Write an object with grit, read it back with git:**

```bash
grit init myrepo
cd myrepo
# (use grit as a library to write a blob — see tests/object_store.rs)
git cat-file blob <oid>    # should print the original content
```

**Write an object with git, read it back with grit:**

```bash
git init myrepo
cd myrepo
echo "hello" | git hash-object -w --stdin   # prints the OID
# (use grit Store::read with that OID — see tests/object_store.rs)
```

The integration tests in `tests/object_store.rs` automate both of these scenarios.

### `grit hash-object` _(not yet implemented)_

Will compute the SHA1 of a file and optionally write it to the object store, equivalent to `git hash-object`.

```bash
grit hash-object file.txt         # print the OID without writing
grit hash-object -w file.txt      # write to the object store and print the OID
```

**Verifying against git:**

```bash
echo "hello" > file.txt
git hash-object file.txt          # should print the same OID as grit
grit hash-object file.txt
```

### `grit cat-file` _(not yet implemented)_

Will read an object from the store by its OID and print its contents or metadata, equivalent to `git cat-file`.

```bash
grit cat-file -t <oid>            # print the object type (blob, tree, commit, tag)
grit cat-file -p <oid>            # pretty-print the object contents
```

**Verifying against git:**

```bash
oid=$(echo "hello" | git hash-object -w --stdin)
git cat-file -p $oid              # should match grit output
grit cat-file -p $oid
```
