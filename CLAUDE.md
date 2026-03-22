# grit — Claude pairing notes

grit is a ground-up reimplementation of git in Rust, built as a learning exercise. The user is growing their Rust knowledge through this project, so explanations of non-obvious Rust patterns are welcome.

## Goals

- Full compatibility with real git: repos created by grit work with git and vice versa
- Implement deeply: object store, index, refs, packfiles, transfer protocol, filesystem interaction
- Follow the git spec rather than reverse-engineering behavior
- Treat this like a real Rust project: idiomatic code, full test coverage, no shortcuts

## Common commands

```bash
cargo build          # build
cargo test           # run all tests
cargo clippy         # lint
cargo fmt            # format
```

## Project structure

```
src/
  main.rs            # thin CLI shell — parses args, calls lib, exits
  lib.rs             # library root — declares modules
  repo.rs            # Repo struct, .git discovery
  refs.rs            # HEAD, branches, tags, packed-refs
  objects/           # git object model and object store
  commands/          # one file per porcelain command (init, add, commit, ...)
  pack/              # packfile read/write (added later)
  transport/         # git wire protocol (added later)
tests/
  common/            # shared test helpers (temp repos, calling git/grit)
```

## Conventions

**Error handling:**
- Library code (`src/`) uses typed errors via `thiserror` — callers can match on variants
- Binary code (`main.rs`) uses `anyhow` for easy propagation and human-readable messages
- Use `.with_context(|| ...)` to add context to errors at I/O boundaries

**Testing:**
- Unit tests live inline in each module under `#[cfg(test)]`
- Integration tests live in `tests/` and test against real git for compatibility verification
- Every public function should have test coverage
- Integration tests use `tempfile::TempDir` for isolation

**Style:**
- Run `cargo fmt` and `cargo clippy` before committing — no warnings left unaddressed
- Prefer explicit error types over `.unwrap()` in library code (`.unwrap()` is acceptable in tests)

## Key specs

- [Git object format](https://git-scm.com/docs/gitformat-object)
- [Pack format](https://git-scm.com/docs/gitformat-pack)
- [Index format](https://git-scm.com/docs/gitformat-index)
- [Protocol v2](https://git-scm.com/docs/protocol-v2)
- [Git source code](https://github.com/git/git) — ground truth when specs are ambiguous

## Current state

- `grit init` — implemented and tested
- Object store (`src/objects/mod.rs`) — fully implemented and tested
  - `ObjectId`, `ObjectKind`, `Object` types with full git object format support
  - `Store::write` — SHA1 hash, zlib-compress, fan-out path layout
  - `Store::read` — decompress and parse from fan-out path
  - Unit tests including known-hash verification against git's SHA1
  - Integration tests verifying grit↔git round-trips in both directions
- Refs, index, remaining commands — stubs only, not yet implemented
