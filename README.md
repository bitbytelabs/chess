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
# Windows binary path:
./target/release/chess.exe
```

## "Train" it to be smarter

This engine does not use neural-network training. Strength comes from improving search/evaluation and validating with tests/benchmarks.

Recommended workflow:

1. **Set a baseline performance**
   ```bash
   ./scripts/bench_nodes_per_sec.sh
   ./scripts/bench_depth_budget.sh

   # Windows (PowerShell)
   ./scripts/bench_nodes_per_sec.ps1
   ./scripts/bench_depth_budget.ps1
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

### Are Stockfish opening books useful for "training"?

Short answer: **helpful for opening play, but not training by themselves**.

- Books like <https://github.com/official-stockfish/books> can improve early-game move choice if you integrate book lookup into move selection.
- They do **not** train the engine's evaluation/search logic on their own.
- For this project, treat a book as an optional opening module; engine strength still mostly comes from search/eval improvements and testing.

### Does `official-stockfish/nnue-pytorch` help?

Short answer: **yes, if you want real ML-style training**.

- <https://github.com/official-stockfish/nnue-pytorch> is a training pipeline for NNUE networks, which is much closer to true model training than opening books.
- To start using it from this repo:
  ```bash
  ./scripts/setup_nnue_pytorch.sh
  ./scripts/train_nnue.sh
  ```
- To benefit in this engine, you would still need NNUE inference support and to replace/augment the current hand-crafted evaluation path.
- Keep validating with tactical tests, perft/movegen safety checks, and engine-vs-engine matches after integration.
