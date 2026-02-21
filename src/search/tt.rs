use std::collections::HashMap;

use crate::engine::board::{Color, PieceType};
use crate::engine::game_state::GameState;

#[derive(Debug, Copy, Clone)]
pub enum NodeType {
    Exact,
    LowerBound,
    UpperBound,
}

#[derive(Debug, Copy, Clone)]
pub struct TTEntry {
    pub depth: i32,
    pub score: i32,
    pub node_type: NodeType,
    pub best_move: Option<crate::engine::board::Move>,
}

pub struct Zobrist {
    piece_keys: [[[u64; 64]; 6]; 2],
    side_to_move: u64,
    castling: [u64; 4],
    en_passant_file: [u64; 8],
}

impl Zobrist {
    pub fn new(seed: u64) -> Self {
        let mut rng = SplitMix64::new(seed);
        let mut piece_keys = [[[0u64; 64]; 6]; 2];
        for color in &mut piece_keys {
            for kind in color {
                for sq in kind {
                    *sq = rng.next();
                }
            }
        }
        let side_to_move = rng.next();
        let castling = [rng.next(), rng.next(), rng.next(), rng.next()];
        let en_passant_file = [
            rng.next(),
            rng.next(),
            rng.next(),
            rng.next(),
            rng.next(),
            rng.next(),
            rng.next(),
            rng.next(),
        ];

        Self {
            piece_keys,
            side_to_move,
            castling,
            en_passant_file,
        }
    }

    pub fn hash(&self, state: &GameState) -> u64 {
        let mut key = 0u64;

        for (sq, piece) in state.board.iter().enumerate() {
            if let Some(p) = piece {
                let c = if p.color == Color::White { 0 } else { 1 };
                let k = match p.kind {
                    PieceType::Pawn => 0,
                    PieceType::Knight => 1,
                    PieceType::Bishop => 2,
                    PieceType::Rook => 3,
                    PieceType::Queen => 4,
                    PieceType::King => 5,
                };
                key ^= self.piece_keys[c][k][sq];
            }
        }

        if state.side_to_move == Color::White {
            key ^= self.side_to_move;
        }
        if state.castling.white_king_side {
            key ^= self.castling[0];
        }
        if state.castling.white_queen_side {
            key ^= self.castling[1];
        }
        if state.castling.black_king_side {
            key ^= self.castling[2];
        }
        if state.castling.black_queen_side {
            key ^= self.castling[3];
        }
        if let Some(ep_sq) = state.en_passant {
            key ^= self.en_passant_file[ep_sq % 8];
        }

        key
    }
}

struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E3779B97F4A7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^ (z >> 31)
    }
}

pub struct TranspositionTable {
    entries: HashMap<u64, TTEntry>,
}

impl TranspositionTable {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    pub fn get(&self, key: u64) -> Option<TTEntry> {
        self.entries.get(&key).copied()
    }

    pub fn insert(&mut self, key: u64, entry: TTEntry) {
        self.entries.insert(key, entry);
    }
}
