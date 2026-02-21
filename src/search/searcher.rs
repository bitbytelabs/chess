use std::time::{Duration, Instant};

use crate::engine::board::Move;
use crate::engine::game_state::GameState;
use crate::engine::legal::{generate_legal_moves, in_check};
use crate::search::eval::evaluate;
use crate::search::move_ordering::order_moves;
use crate::search::tt::{NodeType, TTEntry, TranspositionTable, Zobrist};

const MATE_SCORE: i32 = 100_000;

#[derive(Clone, Debug)]
pub struct SearchConfig {
    pub max_depth: u8,
    pub time_limit_ms: Option<u64>,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            max_depth: 4,
            time_limit_ms: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub best_move: Option<Move>,
    pub score: i32,
    pub depth: u8,
    pub nodes: u64,
}

pub struct Searcher {
    pub config: SearchConfig,
    pub tt: TranspositionTable,
    zobrist: Zobrist,
    start_time: Instant,
    nodes: u64,
}

impl Searcher {
    pub fn new(config: SearchConfig) -> Self {
        Self {
            config,
            tt: TranspositionTable::new(),
            zobrist: Zobrist::new(0xC0FFEE),
            start_time: Instant::now(),
            nodes: 0,
        }
    }

    pub fn search(&mut self, state: &GameState) -> SearchResult {
        self.start_time = Instant::now();
        self.nodes = 0;

        let mut best_move = None;
        let mut best_score = 0;
        let mut reached_depth = 0;

        for depth in 1..=self.config.max_depth {
            if self.time_exceeded() {
                break;
            }

            let (score, mv) = self.root_negamax(state, depth as i32, -MATE_SCORE, MATE_SCORE);
            if self.time_exceeded() {
                break;
            }
            best_score = score;
            best_move = mv;
            reached_depth = depth;
        }

        SearchResult {
            best_move,
            score: best_score,
            depth: reached_depth,
            nodes: self.nodes,
        }
    }

    fn root_negamax(
        &mut self,
        state: &GameState,
        depth: i32,
        mut alpha: i32,
        beta: i32,
    ) -> (i32, Option<Move>) {
        let mut moves = generate_legal_moves(state);
        if moves.is_empty() {
            if in_check(state, state.side_to_move) {
                return (-MATE_SCORE + 1, None);
            }
            return (0, None);
        }

        let key = self.zobrist.hash(state);
        let tt_move = self.tt.get(key).and_then(|e| e.best_move);
        order_moves(state, &mut moves, tt_move);

        let mut best_move = None;
        let mut best_score = -MATE_SCORE;

        for mv in moves {
            if self.time_exceeded() {
                break;
            }
            let mut child = state.clone();
            child.make_move(mv);
            let score = -self.negamax(&child, depth - 1, -beta, -alpha);
            if score > best_score {
                best_score = score;
                best_move = Some(mv);
            }
            alpha = alpha.max(score);
            if alpha >= beta {
                break;
            }
        }

        (best_score, best_move)
    }

    fn negamax(&mut self, state: &GameState, depth: i32, mut alpha: i32, beta: i32) -> i32 {
        if self.time_exceeded() {
            return evaluate(state);
        }

        self.nodes += 1;
        let alpha_orig = alpha;
        let key = self.zobrist.hash(state);

        if let Some(entry) = self.tt.get(key) {
            if entry.depth >= depth {
                match entry.node_type {
                    NodeType::Exact => return entry.score,
                    NodeType::LowerBound => alpha = alpha.max(entry.score),
                    NodeType::UpperBound => {}
                }
                if alpha >= beta {
                    return entry.score;
                }
            }
        }

        if depth <= 0 {
            return self.quiescence(state, alpha, beta);
        }

        let mut moves = generate_legal_moves(state);
        if moves.is_empty() {
            return if in_check(state, state.side_to_move) {
                -MATE_SCORE + 1
            } else {
                0
            };
        }

        let tt_move = self.tt.get(key).and_then(|e| e.best_move);
        order_moves(state, &mut moves, tt_move);

        let mut best_score = -MATE_SCORE;
        let mut best_move = None;

        for mv in moves {
            let mut child = state.clone();
            child.make_move(mv);
            let score = -self.negamax(&child, depth - 1, -beta, -alpha);
            best_score = best_score.max(score);
            if score > alpha {
                alpha = score;
                best_move = Some(mv);
            }
            if alpha >= beta {
                break;
            }
        }

        let node_type = if best_score <= alpha_orig {
            NodeType::UpperBound
        } else if best_score >= beta {
            NodeType::LowerBound
        } else {
            NodeType::Exact
        };
        self.tt.insert(
            key,
            TTEntry {
                depth,
                score: best_score,
                node_type,
                best_move,
            },
        );

        best_score
    }

    fn quiescence(&mut self, state: &GameState, mut alpha: i32, beta: i32) -> i32 {
        self.nodes += 1;
        let stand_pat = evaluate(state);
        if stand_pat >= beta {
            return beta;
        }
        alpha = alpha.max(stand_pat);

        let moves = generate_legal_moves(state);
        for mv in moves
            .into_iter()
            .filter(|m| state.board[m.to].is_some() || m.is_en_passant)
        {
            let mut child = state.clone();
            child.make_move(mv);
            let score = -self.quiescence(&child, -beta, -alpha);
            if score >= beta {
                return beta;
            }
            alpha = alpha.max(score);
        }

        alpha
    }

    fn time_exceeded(&self) -> bool {
        match self.config.time_limit_ms {
            Some(ms) => self.start_time.elapsed() >= Duration::from_millis(ms),
            None => false,
        }
    }
}
