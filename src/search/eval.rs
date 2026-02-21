use crate::engine::board::{Color, PieceType};
use crate::engine::game_state::GameState;

pub fn evaluate(state: &GameState) -> i32 {
    let mut score = 0;
    for piece in state.board.iter().flatten() {
        let value = match piece.kind {
            PieceType::Pawn => 100,
            PieceType::Knight => 320,
            PieceType::Bishop => 330,
            PieceType::Rook => 500,
            PieceType::Queen => 900,
            PieceType::King => 20_000,
        };
        if piece.color == Color::White {
            score += value;
        } else {
            score -= value;
        }
    }

    if state.side_to_move == Color::White {
        score
    } else {
        -score
    }
}
