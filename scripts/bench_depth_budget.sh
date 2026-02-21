#!/usr/bin/env bash
set -euo pipefail

cargo test --release benchmark_depth_reached_by_time_budget --test engine_suite -- --ignored --nocapture
