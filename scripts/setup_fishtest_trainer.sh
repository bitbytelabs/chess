#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
VENV_DIR="$ROOT_DIR/.venv-fishtest"

python3 -m venv "$VENV_DIR"
source "$VENV_DIR/bin/activate"
python -m pip install --upgrade pip
python -m pip install python-chess

echo ""
echo "Local fishtest-like trainer is ready."
echo "Activate with: source $VENV_DIR/bin/activate"
echo "Run: ./scripts/fishtest_like_trainer.py --candidate ./target/release/chess --baseline ./target/release/chess"
