use chess::engine::fen::parse_fen;
use chess::engine::legal::generate_legal_moves;
use chess::search::searcher::{SearchConfig, Searcher};

#[test]
fn startpos_has_20_legal_moves() {
    let state = parse_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
    assert_eq!(generate_legal_moves(&state).len(), 20);
}

#[test]
fn search_returns_a_move() {
    let state = parse_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
    let mut searcher = Searcher::new(SearchConfig {
        max_depth: 2,
        time_limit_ms: None,
    });
    let result = searcher.search(&state);
    assert!(result.best_move.is_some());
}
