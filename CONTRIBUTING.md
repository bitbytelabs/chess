# Contributing

## Commit message requirements

This repository uses Conventional Commits and automated semantic versioning in CI.

Version bump rules:

- `fix: ...` → **PATCH** (`1.5.3` → `1.5.4`)
- `feat: ...` → **MINOR** (`1.5.3` → `1.6.0`)
- `feat!: ...` or commit body containing `BREAKING CHANGE` → **MAJOR** (`1.5.3` → `2.0.0`)

Use commit messages in this exact style:

- `fix: correct typo`
- `feat: add login system`
- `feat!: change API structure`

These formats are required because the GitHub Action parses commit history to bump `VERSION`, sync `Cargo.toml`, and regenerate `CHANGELOG.md` automatically.

## Test and merge requirements

All pull requests must pass the `CI / test` workflow before merge. The workflow runs:

- `cargo fmt --all --check`
- `cargo test --all-targets --all-features`

Engine benchmark scripts are available for local performance tracking:

- `./scripts/bench_nodes_per_sec.sh`
- `./scripts/bench_depth_budget.sh`
