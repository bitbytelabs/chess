#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
NNUE_DIR="$ROOT_DIR/external/nnue-pytorch"
NNUE_REPO="https://github.com/official-stockfish/nnue-pytorch"

if [[ -d "$NNUE_DIR/.git" ]]; then
  echo "nnue-pytorch already present at $NNUE_DIR"
  exit 0
fi

mkdir -p "$ROOT_DIR/external"
git clone "$NNUE_REPO" "$NNUE_DIR"
echo "Cloned nnue-pytorch into $NNUE_DIR"
