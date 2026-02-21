use std::io::{self, BufRead, Write};

use crate::engine::board::{str_to_square, Move, PieceType};
use crate::engine::fen::{parse_fen, STARTPOS_FEN};
use crate::engine::game_state::GameState;
use crate::engine::legal::generate_legal_moves;
use crate::search::searcher::{SearchConfig, Searcher};

pub struct UciEngine {
    state: GameState,
    searcher: Searcher,
}

impl UciEngine {
    pub fn new(config: SearchConfig) -> Self {
        let state = parse_fen(STARTPOS_FEN).expect("valid start position FEN");
        Self {
            state,
            searcher: Searcher::new(config),
        }
    }

    pub fn run(&mut self) {
        let stdin = io::stdin();
        let mut stdout = io::stdout();

        for line in stdin.lock().lines() {
            let Ok(line) = line else {
                break;
            };
            if !self.handle_command(line.trim(), &mut stdout) {
                break;
            }
        }
    }

    fn handle_command(&mut self, line: &str, stdout: &mut impl Write) -> bool {
        if line == "uci" {
            let _ = writeln!(stdout, "id name ChessBaseline");
            let _ = writeln!(stdout, "id author Codex");
            let _ = writeln!(stdout, "uciok");
        } else if line == "isready" {
            let _ = writeln!(stdout, "readyok");
        } else if line.starts_with("position") {
            self.apply_position(line);
        } else if line.starts_with("go") {
            self.apply_go(line);
            let result = self.searcher.search(&self.state);
            if let Some(best) = result.best_move {
                let _ = writeln!(
                    stdout,
                    "info depth {} score cp {} nodes {}",
                    result.depth, result.score, result.nodes
                );
                let _ = writeln!(stdout, "bestmove {}", best.to_uci());
            } else {
                let _ = writeln!(stdout, "bestmove 0000");
            }
        } else if line == "ucinewgame" {
            self.state = parse_fen(STARTPOS_FEN).expect("valid start position FEN");
        } else if line == "quit" {
            return false;
        }

        let _ = stdout.flush();
        true
    }

    fn apply_go(&mut self, line: &str) {
        let mut max_depth = self.searcher.config.max_depth;
        let mut time_limit_ms = self.searcher.config.time_limit_ms;
        let parts: Vec<&str> = line.split_whitespace().collect();

        let mut i = 1usize;
        while i < parts.len() {
            match parts[i] {
                "depth" if i + 1 < parts.len() => {
                    if let Ok(depth) = parts[i + 1].parse::<u8>() {
                        max_depth = depth;
                    }
                    i += 1;
                }
                "movetime" if i + 1 < parts.len() => {
                    if let Ok(ms) = parts[i + 1].parse::<u64>() {
                        time_limit_ms = Some(ms);
                    }
                    i += 1;
                }
                _ => {}
            }
            i += 1;
        }

        self.searcher.config.max_depth = max_depth;
        self.searcher.config.time_limit_ms = time_limit_ms;
    }

    fn apply_position(&mut self, line: &str) {
        let mut parts = line.split_whitespace();
        let _ = parts.next();

        let mut state = if let Some(kind) = parts.next() {
            if kind == "startpos" {
                parse_fen(STARTPOS_FEN).expect("valid start position")
            } else if kind == "fen" {
                let fen_fields: Vec<&str> = parts.by_ref().take(6).collect();
                if fen_fields.len() != 6 {
                    return;
                }
                parse_fen(&fen_fields.join(" "))
                    .unwrap_or_else(|_| parse_fen(STARTPOS_FEN).expect("valid start position"))
            } else {
                return;
            }
        } else {
            return;
        };

        let rest: Vec<&str> = parts.collect();
        if let Some(pos) = rest.iter().position(|p| *p == "moves") {
            for mv_str in &rest[pos + 1..] {
                if let Some(mv) = parse_uci_move(&state, mv_str) {
                    state.make_move(mv);
                }
            }
        }

        self.state = state;
    }
}

fn parse_uci_move(state: &GameState, text: &str) -> Option<Move> {
    if text.len() < 4 {
        return None;
    }
    let from = str_to_square(&text[0..2])?;
    let to = str_to_square(&text[2..4])?;
    let promotion = if text.len() == 5 {
        match &text[4..5] {
            "n" => Some(PieceType::Knight),
            "b" => Some(PieceType::Bishop),
            "r" => Some(PieceType::Rook),
            "q" => Some(PieceType::Queen),
            _ => None,
        }
    } else {
        None
    };

    generate_legal_moves(state)
        .into_iter()
        .find(|m| m.from == from && m.to == to && m.promotion == promotion)
}
