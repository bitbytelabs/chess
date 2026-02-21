# Chess Engine (Rust)

A small UCI-compatible chess engine written in Rust.

## Run the engine

### 1) Build and start

```bash
cargo run -- --depth 6
```

Optional flags:

- `--depth N` sets a fixed max search depth (default: `4`)
- `--movetime-ms MS` sets a per-move time budget in milliseconds

Examples:

```bash
cargo run -- --depth 8
cargo run -- --movetime-ms 500
cargo run -- --depth 10 --movetime-ms 250
```

### 2) Talk to it in UCI mode

Once running, send UCI commands:

```text
uci
isready
position startpos
go depth 5
quit
```

Quick non-interactive test:

```bash
printf "uci\nisready\nposition startpos\ngo depth 4\nquit\n" | cargo run -- --depth 4
```

## Use it with a chess GUI

Because it speaks UCI, you can load it in GUIs like Arena, Cute Chess, or Banksia.
Point the GUI engine path to this binary:

```bash
cargo build --release
# binary path:
./target/release/chess
```

## "Train" it to be smarter

This engine does not use neural-network training. Strength comes from improving search/evaluation and validating with tests/benchmarks.

Recommended workflow:

1. **Set a baseline performance**
   ```bash
   ./scripts/bench_nodes_per_sec.sh
   ./scripts/bench_depth_budget.sh
   ```
2. **Improve evaluation/search**
   - Tune material values in `src/search/eval.rs`
   - Improve move ordering and pruning in `src/search/`
3. **Add regression tests for tactics/positions**
   - Add positions to `tests/engine/search.rs`
   - Keep perft/movegen tests passing to avoid illegal move regressions
4. **Re-run full validation**
   ```bash
   cargo fmt --all --check
   cargo test --all-targets --all-features
   ```

If you want true ML-style training, you would need to add a separate self-play data pipeline and a learned evaluation model (not present in this repo today).
