$ErrorActionPreference = 'Stop'

cargo test --release benchmark_nodes_per_second_fixed_budget --test engine_suite -- --ignored --nocapture
