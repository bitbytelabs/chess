use crate::engine::board::{file_of, rank_of, square, Color, PieceType};
use crate::engine::game_state::GameState;
use crate::engine::movegen::generate_pseudo_legal_moves;

pub fn is_square_attacked(state: &GameState, sq: usize, by_color: Color) -> bool {
    for from in 0..64 {
        let Some(piece) = state.board[from] else {
            continue;
        };
        if piece.color != by_color {
            continue;
        }

        let f = file_of(from) as isize;
        let r = rank_of(from) as isize;
        match piece.kind {
            PieceType::Pawn => {
                let dir: isize = if by_color == Color::White { 1 } else { -1 };
                for df in [-1, 1] {
                    let tf = f + df;
                    let tr = r + dir;
                    if (0..8).contains(&tf)
                        && (0..8).contains(&tr)
                        && square(tf as usize, tr as usize) == sq
                    {
                        return true;
                    }
                }
            }
            PieceType::Knight => {
                for (df, dr) in [
                    (1, 2),
                    (2, 1),
                    (2, -1),
                    (1, -2),
                    (-1, -2),
                    (-2, -1),
                    (-2, 1),
                    (-1, 2),
                ] {
                    let tf = f + df;
                    let tr = r + dr;
                    if (0..8).contains(&tf)
                        && (0..8).contains(&tr)
                        && square(tf as usize, tr as usize) == sq
                    {
                        return true;
                    }
                }
            }
            PieceType::Bishop => {
                if ray_attacks(state, from, sq, &[(1, 1), (1, -1), (-1, 1), (-1, -1)]) {
                    return true;
                }
            }
            PieceType::Rook => {
                if ray_attacks(state, from, sq, &[(1, 0), (-1, 0), (0, 1), (0, -1)]) {
                    return true;
                }
            }
            PieceType::Queen => {
                if ray_attacks(
                    state,
                    from,
                    sq,
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
                ) {
                    return true;
                }
            }
            PieceType::King => {
                for df in -1..=1 {
                    for dr in -1..=1 {
                        if df == 0 && dr == 0 {
                            continue;
                        }
                        let tf = f + df;
                        let tr = r + dr;
                        if (0..8).contains(&tf)
                            && (0..8).contains(&tr)
                            && square(tf as usize, tr as usize) == sq
                        {
                            return true;
                        }
                    }
                }
            }
        }
    }
    false
}

fn ray_attacks(state: &GameState, from: usize, target: usize, dirs: &[(isize, isize)]) -> bool {
    for (df, dr) in dirs {
        let mut tf = file_of(from) as isize + df;
        let mut tr = rank_of(from) as isize + dr;
        while (0..8).contains(&tf) && (0..8).contains(&tr) {
            let sq = square(tf as usize, tr as usize);
            if sq == target {
                return true;
            }
            if state.board[sq].is_some() {
                break;
            }
            tf += df;
            tr += dr;
        }
    }
    false
}

pub fn in_check(state: &GameState, color: Color) -> bool {
    let Some(king_sq) = state.king_square(color) else {
        return false;
    };
    is_square_attacked(state, king_sq, color.opposite())
}

pub fn generate_legal_moves(state: &GameState) -> Vec<crate::engine::board::Move> {
    let mut legal = Vec::new();
    for mv in generate_pseudo_legal_moves(state) {
        let mut next = state.clone();
        next.make_move(mv);

        if mv.is_castling {
            let (k_from, k_mid, k_to) = if state.side_to_move == Color::White {
                if mv.to == 6 {
                    (4, 5, 6)
                } else {
                    (4, 3, 2)
                }
            } else if mv.to == 62 {
                (60, 61, 62)
            } else {
                (60, 59, 58)
            };
            if is_square_attacked(state, k_from, state.side_to_move.opposite())
                || is_square_attacked(state, k_mid, state.side_to_move.opposite())
                || is_square_attacked(&next, k_to, state.side_to_move.opposite())
            {
                continue;
            }
        }

        if !in_check(&next, state.side_to_move) {
            legal.push(mv);
        }
    }
    legal
}
