#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
NNUE_DIR="$ROOT_DIR/external/nnue-pytorch"
VENV_DIR="$NNUE_DIR/.venv"

if [[ ! -d "$NNUE_DIR" ]]; then
  echo "Missing $NNUE_DIR. Run ./scripts/setup_nnue_pytorch.sh first." >&2
  exit 1
fi

python3 -m venv "$VENV_DIR"
source "$VENV_DIR/bin/activate"
python -m pip install --upgrade pip
python -m pip install -r "$NNUE_DIR/requirements.txt"

cat <<'USAGE'

nnue-pytorch is installed and ready.

Next, run training using the upstream scripts, for example:
  cd external/nnue-pytorch
  python train.py --help

After you produce a network, integrate inference in this engine before expecting Elo gains.
USAGE
