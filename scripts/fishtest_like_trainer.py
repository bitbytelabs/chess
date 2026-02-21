#!/usr/bin/env python3
"""Run local, fishtest-like engine matches between baseline and candidate binaries.

This script is intentionally lightweight and downloadable: it only requires python-chess.
It launches two UCI engines, feeds random opening positions, and tracks WDL statistics.
"""

from __future__ import annotations

import argparse
import math
import random
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path

import chess
import chess.engine


@dataclass
class Score:
    wins: int = 0
    draws: int = 0
    losses: int = 0

    def record(self, result_for_candidate: float) -> None:
        if result_for_candidate == 1.0:
            self.wins += 1
        elif result_for_candidate == 0.5:
            self.draws += 1
        else:
            self.losses += 1

    @property
    def games(self) -> int:
        return self.wins + self.draws + self.losses

    @property
    def score_rate(self) -> float:
        if self.games == 0:
            return 0.5
        return (self.wins + 0.5 * self.draws) / self.games

    @property
    def elo_estimate(self) -> float:
        p = min(max(self.score_rate, 1e-6), 1 - 1e-6)
        return -400.0 * math.log10((1.0 / p) - 1.0)



def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Local fishtest-style match runner")
    parser.add_argument("--candidate", required=True, help="Path to candidate engine binary")
    parser.add_argument("--baseline", required=True, help="Path to baseline engine binary")
    parser.add_argument("--games", type=int, default=20, help="Total number of games")
    parser.add_argument("--movetime-ms", type=int, default=100, help="Time per move in ms")
    parser.add_argument("--max-plies", type=int, default=200, help="Max plies before draw")
    parser.add_argument(
        "--opening-random-plies",
        type=int,
        default=6,
        help="Random opening plies before engines take over",
    )
    parser.add_argument("--seed", type=int, default=42, help="PRNG seed")
    return parser.parse_args()


def random_opening_board(rng: random.Random, plies: int) -> chess.Board:
    board = chess.Board()
    for _ in range(plies):
        if board.is_game_over():
            break
        moves = list(board.legal_moves)
        if not moves:
            break
        board.push(rng.choice(moves))
    return board


def ensure_binary(path: str) -> Path:
    p = Path(path)
    if not p.exists():
        raise FileNotFoundError(f"Engine not found: {path}")
    if not p.is_file():
        raise FileNotFoundError(f"Not a file: {path}")
    return p


def play_game(
    white_engine: chess.engine.SimpleEngine,
    black_engine: chess.engine.SimpleEngine,
    movetime_ms: int,
    max_plies: int,
    opening_board: chess.Board,
) -> str:
    board = opening_board.copy(stack=False)
    limit = chess.engine.Limit(time=movetime_ms / 1000.0)

    for _ in range(max_plies):
        if board.is_game_over(claim_draw=True):
            break

        engine = white_engine if board.turn == chess.WHITE else black_engine
        result = engine.play(board, limit)
        board.push(result.move)

    if not board.is_game_over(claim_draw=True):
        return "1/2-1/2"

    outcome = board.outcome(claim_draw=True)
    if outcome is None or outcome.winner is None:
        return "1/2-1/2"
    return "1-0" if outcome.winner == chess.WHITE else "0-1"


def result_for_candidate(result: str, candidate_is_white: bool) -> float:
    if result == "1/2-1/2":
        return 0.5
    if result == "1-0":
        return 1.0 if candidate_is_white else 0.0
    return 0.0 if candidate_is_white else 1.0


def main() -> int:
    args = parse_args()
    rng = random.Random(args.seed)

    candidate_path = ensure_binary(args.candidate)
    baseline_path = ensure_binary(args.baseline)

    score = Score()

    with chess.engine.SimpleEngine.popen_uci(str(candidate_path)) as candidate_engine, chess.engine.SimpleEngine.popen_uci(
        str(baseline_path)
    ) as baseline_engine:
        for game_index in range(args.games):
            candidate_is_white = game_index % 2 == 0
            white = candidate_engine if candidate_is_white else baseline_engine
            black = baseline_engine if candidate_is_white else candidate_engine

            opening = random_opening_board(rng, args.opening_random_plies)
            result = play_game(white, black, args.movetime_ms, args.max_plies, opening)
            score.record(result_for_candidate(result, candidate_is_white))

            print(
                f"game {game_index + 1}/{args.games}: result={result} "
                f"candidate_score={score.wins}-{score.draws}-{score.losses} "
                f"elo_est={score.elo_estimate:.1f}"
            )

    print("\n=== Summary ===")
    print(f"Candidate W-D-L: {score.wins}-{score.draws}-{score.losses}")
    print(f"Score rate: {score.score_rate:.3f}")
    print(f"Estimated Elo delta: {score.elo_estimate:.1f}")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except subprocess.SubprocessError as exc:
        print(f"Engine process error: {exc}", file=sys.stderr)
        raise SystemExit(2)
    except FileNotFoundError as exc:
        print(str(exc), file=sys.stderr)
        raise SystemExit(2)
