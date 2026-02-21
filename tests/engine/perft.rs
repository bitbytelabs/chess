use chess::engine::fen::{parse_fen, STARTPOS_FEN};
use chess::engine::game_state::GameState;
use chess::engine::legal::generate_legal_moves;

fn perft(state: &GameState, depth: u8) -> u64 {
    if depth == 0 {
        return 1;
    }

    let moves = generate_legal_moves(state);
    if depth == 1 {
        return moves.len() as u64;
    }

    let mut nodes = 0;
    for mv in moves {
        let mut child = state.clone();
        child.make_move(mv);
        nodes += perft(&child, depth - 1);
    }
    nodes
}

#[test]
fn start_position_perft_regression() {
    let state = parse_fen(STARTPOS_FEN).unwrap();

    assert_eq!(perft(&state, 1), 20);
    assert_eq!(perft(&state, 2), 400);
    assert_eq!(perft(&state, 3), 8_902);
}

#[test]
fn king_vs_king_corner_perft_regression() {
    let state = parse_fen("7k/8/8/8/8/8/8/K7 w - - 0 1").unwrap();

    assert_eq!(perft(&state, 1), 3);
    assert_eq!(perft(&state, 2), 9);
    assert_eq!(perft(&state, 3), 54);
}
