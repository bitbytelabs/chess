use crate::engine::board::{file_of, rank_of, square, Color, Move, Piece, PieceType};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct CastlingRights {
    pub white_king_side: bool,
    pub white_queen_side: bool,
    pub black_king_side: bool,
    pub black_queen_side: bool,
}

impl Default for CastlingRights {
    fn default() -> Self {
        Self {
            white_king_side: true,
            white_queen_side: true,
            black_king_side: true,
            black_queen_side: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct GameState {
    pub board: [Option<Piece>; 64],
    pub side_to_move: Color,
    pub castling: CastlingRights,
    pub en_passant: Option<usize>,
    pub halfmove_clock: u32,
    pub fullmove_number: u32,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            board: [None; 64],
            side_to_move: Color::White,
            castling: CastlingRights::default(),
            en_passant: None,
            halfmove_clock: 0,
            fullmove_number: 1,
        }
    }
}

impl GameState {
    pub fn piece_at(&self, sq: usize) -> Option<Piece> {
        self.board[sq]
    }

    pub fn king_square(&self, color: Color) -> Option<usize> {
        self.board.iter().enumerate().find_map(|(sq, p)| match p {
            Some(piece) if piece.color == color && piece.kind == PieceType::King => Some(sq),
            _ => None,
        })
    }

    pub fn make_move(&mut self, mv: Move) {
        let moving_piece = self.board[mv.from];
        if moving_piece.is_none() {
            return;
        }
        let mut piece = moving_piece.unwrap();

        if mv.is_en_passant {
            let capture_rank = if piece.color == Color::White {
                rank_of(mv.to).saturating_sub(1)
            } else {
                rank_of(mv.to) + 1
            };
            let capture_sq = square(file_of(mv.to), capture_rank);
            self.board[capture_sq] = None;
        }

        self.board[mv.from] = None;
        if let Some(promoted) = mv.promotion {
            piece.kind = promoted;
        }
        self.board[mv.to] = Some(piece);

        if mv.is_castling {
            match (piece.color, mv.to) {
                (Color::White, 6) => {
                    self.board[7] = None;
                    self.board[5] = Some(Piece {
                        color: Color::White,
                        kind: PieceType::Rook,
                    });
                }
                (Color::White, 2) => {
                    self.board[0] = None;
                    self.board[3] = Some(Piece {
                        color: Color::White,
                        kind: PieceType::Rook,
                    });
                }
                (Color::Black, 62) => {
                    self.board[63] = None;
                    self.board[61] = Some(Piece {
                        color: Color::Black,
                        kind: PieceType::Rook,
                    });
                }
                (Color::Black, 58) => {
                    self.board[56] = None;
                    self.board[59] = Some(Piece {
                        color: Color::Black,
                        kind: PieceType::Rook,
                    });
                }
                _ => {}
            }
        }

        self.update_castling_rights(piece, mv);
        self.update_en_passant(piece, mv);

        self.side_to_move = self.side_to_move.opposite();
        if self.side_to_move == Color::White {
            self.fullmove_number += 1;
        }
    }

    fn update_castling_rights(&mut self, piece: Piece, mv: Move) {
        match piece.kind {
            PieceType::King => match piece.color {
                Color::White => {
                    self.castling.white_king_side = false;
                    self.castling.white_queen_side = false;
                }
                Color::Black => {
                    self.castling.black_king_side = false;
                    self.castling.black_queen_side = false;
                }
            },
            PieceType::Rook => match mv.from {
                0 => self.castling.white_queen_side = false,
                7 => self.castling.white_king_side = false,
                56 => self.castling.black_queen_side = false,
                63 => self.castling.black_king_side = false,
                _ => {}
            },
            _ => {}
        }

        match mv.to {
            0 => self.castling.white_queen_side = false,
            7 => self.castling.white_king_side = false,
            56 => self.castling.black_queen_side = false,
            63 => self.castling.black_king_side = false,
            _ => {}
        }
    }

    fn update_en_passant(&mut self, piece: Piece, mv: Move) {
        self.en_passant = None;
        if piece.kind == PieceType::Pawn {
            let from_rank = rank_of(mv.from);
            let to_rank = rank_of(mv.to);
            if from_rank.abs_diff(to_rank) == 2 {
                let ep_rank = (from_rank + to_rank) / 2;
                self.en_passant = Some(square(file_of(mv.from), ep_rank));
            }
        }
    }
}
