use crate::engine::board::{file_of, rank_of, square, Color, Move, PieceType};
use crate::engine::game_state::GameState;

pub fn generate_pseudo_legal_moves(state: &GameState) -> Vec<Move> {
    let mut moves = Vec::with_capacity(64);
    for sq in 0..64 {
        let Some(piece) = state.board[sq] else {
            continue;
        };
        if piece.color != state.side_to_move {
            continue;
        }
        match piece.kind {
            PieceType::Pawn => gen_pawn_moves(state, sq, &mut moves),
            PieceType::Knight => gen_knight_moves(state, sq, &mut moves),
            PieceType::Bishop => {
                gen_sliding_moves(state, sq, &mut moves, &[(1, 1), (1, -1), (-1, 1), (-1, -1)])
            }
            PieceType::Rook => {
                gen_sliding_moves(state, sq, &mut moves, &[(1, 0), (-1, 0), (0, 1), (0, -1)])
            }
            PieceType::Queen => gen_sliding_moves(
                state,
                sq,
                &mut moves,
                &[
                    (1, 1),
                    (1, -1),
                    (-1, 1),
                    (-1, -1),
                    (1, 0),
                    (-1, 0),
                    (0, 1),
                    (0, -1),
                ],
            ),
            PieceType::King => {
                gen_king_moves(state, sq, &mut moves);
                gen_castling_moves(state, sq, &mut moves);
            }
        }
    }
    moves
}

fn gen_pawn_moves(state: &GameState, from: usize, out: &mut Vec<Move>) {
    let piece = state.board[from].unwrap();
    let dir: isize = if piece.color == Color::White { 1 } else { -1 };
    let start_rank = if piece.color == Color::White { 1 } else { 6 };
    let promotion_rank = if piece.color == Color::White { 7 } else { 0 };

    let file = file_of(from) as isize;
    let rank = rank_of(from) as isize;

    let one_step_rank = rank + dir;
    if (0..8).contains(&one_step_rank) {
        let to = square(file as usize, one_step_rank as usize);
        if state.board[to].is_none() {
            if one_step_rank as usize == promotion_rank {
                for promotion in [
                    PieceType::Queen,
                    PieceType::Rook,
                    PieceType::Bishop,
                    PieceType::Knight,
                ] {
                    out.push(Move {
                        promotion: Some(promotion),
                        ..Move::new(from, to)
                    });
                }
            } else {
                out.push(Move::new(from, to));
            }

            if rank as usize == start_rank {
                let two_step_rank = rank + 2 * dir;
                let two_step = square(file as usize, two_step_rank as usize);
                if state.board[two_step].is_none() {
                    out.push(Move::new(from, two_step));
                }
            }
        }
    }

    for df in [-1, 1] {
        let target_file = file + df;
        let target_rank = rank + dir;
        if !(0..8).contains(&target_file) || !(0..8).contains(&target_rank) {
            continue;
        }
        let to = square(target_file as usize, target_rank as usize);
        if let Some(capture) = state.board[to] {
            if capture.color != piece.color {
                if target_rank as usize == promotion_rank {
                    for promotion in [
                        PieceType::Queen,
                        PieceType::Rook,
                        PieceType::Bishop,
                        PieceType::Knight,
                    ] {
                        out.push(Move {
                            promotion: Some(promotion),
                            ..Move::new(from, to)
                        });
                    }
                } else {
                    out.push(Move::new(from, to));
                }
            }
        } else if state.en_passant == Some(to) {
            out.push(Move {
                is_en_passant: true,
                ..Move::new(from, to)
            });
        }
    }
}

fn gen_knight_moves(state: &GameState, from: usize, out: &mut Vec<Move>) {
    let piece = state.board[from].unwrap();
    let file = file_of(from) as isize;
    let rank = rank_of(from) as isize;
    const OFFSETS: [(isize, isize); 8] = [
        (1, 2),
        (2, 1),
        (2, -1),
        (1, -2),
        (-1, -2),
        (-2, -1),
        (-2, 1),
        (-1, 2),
    ];

    for (df, dr) in OFFSETS {
        let tf = file + df;
        let tr = rank + dr;
        if !(0..8).contains(&tf) || !(0..8).contains(&tr) {
            continue;
        }
        let to = square(tf as usize, tr as usize);
        if state.board[to].is_none_or(|p| p.color != piece.color) {
            out.push(Move::new(from, to));
        }
    }
}

fn gen_sliding_moves(
    state: &GameState,
    from: usize,
    out: &mut Vec<Move>,
    directions: &[(isize, isize)],
) {
    let piece = state.board[from].unwrap();
    let file = file_of(from) as isize;
    let rank = rank_of(from) as isize;

    for (df, dr) in directions {
        let mut tf = file + df;
        let mut tr = rank + dr;
        while (0..8).contains(&tf) && (0..8).contains(&tr) {
            let to = square(tf as usize, tr as usize);
            if let Some(target) = state.board[to] {
                if target.color != piece.color {
                    out.push(Move::new(from, to));
                }
                break;
            } else {
                out.push(Move::new(from, to));
            }
            tf += df;
            tr += dr;
        }
    }
}

fn gen_king_moves(state: &GameState, from: usize, out: &mut Vec<Move>) {
    let piece = state.board[from].unwrap();
    for df in -1..=1 {
        for dr in -1..=1 {
            if df == 0 && dr == 0 {
                continue;
            }
            let tf = file_of(from) as isize + df;
            let tr = rank_of(from) as isize + dr;
            if !(0..8).contains(&tf) || !(0..8).contains(&tr) {
                continue;
            }
            let to = square(tf as usize, tr as usize);
            if state.board[to].is_none_or(|p| p.color != piece.color) {
                out.push(Move::new(from, to));
            }
        }
    }
}

fn gen_castling_moves(state: &GameState, from: usize, out: &mut Vec<Move>) {
    let piece = state.board[from].unwrap();
    if piece.kind != PieceType::King {
        return;
    }

    match piece.color {
        Color::White => {
            if state.castling.white_king_side
                && state.board[5].is_none()
                && state.board[6].is_none()
            {
                out.push(Move {
                    is_castling: true,
                    ..Move::new(from, 6)
                });
            }
            if state.castling.white_queen_side
                && state.board[1].is_none()
                && state.board[2].is_none()
                && state.board[3].is_none()
            {
                out.push(Move {
                    is_castling: true,
                    ..Move::new(from, 2)
                });
            }
        }
        Color::Black => {
            if state.castling.black_king_side
                && state.board[61].is_none()
                && state.board[62].is_none()
            {
                out.push(Move {
                    is_castling: true,
                    ..Move::new(from, 62)
                });
            }
            if state.castling.black_queen_side
                && state.board[57].is_none()
                && state.board[58].is_none()
                && state.board[59].is_none()
            {
                out.push(Move {
                    is_castling: true,
                    ..Move::new(from, 58)
                });
            }
        }
    }
}
