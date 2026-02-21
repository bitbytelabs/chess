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

### How to use this in practice

1. Pick the right commit type for the change:
   - Bug fix with no breaking changes: `fix:`
   - New backward-compatible feature: `feat:`
   - Breaking API/behavior change: `feat!:` (or include `BREAKING CHANGE` in the body)
2. Write the commit as `<type>: <short description>`.
3. Keep the description imperative and concise (for example, `fix: handle missing config file`).
4. Push your branch; the release workflow will determine the version bump from your commit history.

Example flow:

```bash
git add .
git commit -m "fix: correct typo"
git push
```

## Test and merge requirements

All pull requests must pass the `CI / test` workflow before merge. The workflow runs:

- `cargo fmt --all --check`
- `cargo test --all-targets --all-features`

Engine benchmark scripts are available for local performance tracking:

- `./scripts/bench_nodes_per_sec.sh`
- `./scripts/bench_depth_budget.sh`
- `./scripts/bench_nodes_per_sec.ps1` (Windows PowerShell)
- `./scripts/bench_depth_budget.ps1` (Windows PowerShell)
