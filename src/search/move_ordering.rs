use crate::engine::board::Move;
use crate::engine::game_state::GameState;

pub fn order_moves(state: &GameState, moves: &mut [Move], tt_move: Option<Move>) {
    moves.sort_by_key(|mv| -score_move(state, *mv, tt_move));
}

fn score_move(state: &GameState, mv: Move, tt_move: Option<Move>) -> i32 {
    if tt_move == Some(mv) {
        return 100_000;
    }

    let victim = state.board[mv.to];
    let attacker = state.board[mv.from];
    if let (Some(victim), Some(attacker)) = (victim, attacker) {
        let victim_value = piece_value(victim.kind);
        let attacker_value = piece_value(attacker.kind);
        return 10_000 + victim_value - attacker_value;
    }

    0
}

fn piece_value(piece: crate::engine::board::PieceType) -> i32 {
    match piece {
        crate::engine::board::PieceType::Pawn => 100,
        crate::engine::board::PieceType::Knight => 320,
        crate::engine::board::PieceType::Bishop => 330,
        crate::engine::board::PieceType::Rook => 500,
        crate::engine::board::PieceType::Queen => 900,
        crate::engine::board::PieceType::King => 20_000,
    }
}
