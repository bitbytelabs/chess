use chess::engine::board::{str_to_square, PieceType};
use chess::engine::fen::parse_fen;
use chess::engine::legal::generate_legal_moves;

fn has_uci_move(moves: &[chess::engine::board::Move], uci: &str) -> bool {
    moves.iter().any(|mv| mv.to_uci() == uci)
}

#[test]
fn castling_is_generated_when_legal() {
    let state = parse_fen("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1").unwrap();
    let moves = generate_legal_moves(&state);

    assert!(has_uci_move(&moves, "e1g1"));
    assert!(has_uci_move(&moves, "e1c1"));
}

#[test]
fn castling_is_blocked_when_king_would_cross_check() {
    let state = parse_fen("r3k2r/8/8/8/2b5/8/8/R3K2R w KQkq - 0 1").unwrap();
    let moves = generate_legal_moves(&state);

    assert!(!has_uci_move(&moves, "e1g1"));
    assert!(has_uci_move(&moves, "e1c1"));
}

#[test]
fn en_passant_capture_is_generated() {
    let state = parse_fen("8/8/8/3pP3/8/8/8/4K2k w - d6 0 1").unwrap();
    let moves = generate_legal_moves(&state);

    assert!(has_uci_move(&moves, "e5d6"));
}

#[test]
fn promotion_options_are_generated() {
    let state = parse_fen("4k3/P7/8/8/8/8/8/4K3 w - - 0 1").unwrap();
    let moves = generate_legal_moves(&state);

    let from = str_to_square("a7").unwrap();
    let to = str_to_square("a8").unwrap();
    let promotions: Vec<PieceType> = moves
        .iter()
        .filter(|mv| mv.from == from && mv.to == to)
        .filter_map(|mv| mv.promotion)
        .collect();

    assert_eq!(promotions.len(), 4);
    assert!(promotions.contains(&PieceType::Queen));
    assert!(promotions.contains(&PieceType::Rook));
    assert!(promotions.contains(&PieceType::Bishop));
    assert!(promotions.contains(&PieceType::Knight));
}

#[test]
fn pinned_piece_cannot_move_exposing_king() {
    let state = parse_fen("4r1k1/8/8/8/8/8/4R3/4K3 w - - 0 1").unwrap();
    let moves = generate_legal_moves(&state);

    assert!(!has_uci_move(&moves, "e2f2"));
    assert!(!has_uci_move(&moves, "e2d2"));
    assert!(has_uci_move(&moves, "e2e8"));
}
