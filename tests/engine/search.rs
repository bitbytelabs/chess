use chess::engine::fen::parse_fen;
use chess::search::eval::evaluate;
use chess::search::searcher::{SearchConfig, Searcher};

#[test]
fn search_finds_mate_in_one() {
    let state = parse_fen("7k/5Q2/6K1/8/8/8/8/8 w - - 0 1").unwrap();
    let mut searcher = Searcher::new(SearchConfig {
        max_depth: 3,
        time_limit_ms: None,
    });

    let result = searcher.search(&state);

    let best = result.best_move.expect("expected a mating move").to_uci();
    assert!(
        result.score > 90_000,
        "expected mate score, got {}",
        result.score
    );
    assert!(matches!(best.as_str(), "f7g7" | "f7f8" | "f7e8"));
}

#[test]
fn search_prefers_winning_major_material_tactic() {
    let state = parse_fen("4k3/8/8/8/8/8/4q3/4R1K1 w - - 0 1").unwrap();
    let mut searcher = Searcher::new(SearchConfig {
        max_depth: 3,
        time_limit_ms: None,
    });

    let result = searcher.search(&state);

    assert_eq!(
        result.best_move.expect("expected a best move").to_uci(),
        "e1e2"
    );
}

#[test]
fn evaluation_increases_with_extra_material_for_side_to_move() {
    let balanced = parse_fen("4k3/8/8/8/8/8/8/4K3 w - - 0 1").unwrap();
    let white_extra_queen = parse_fen("4k3/8/8/8/8/8/8/3QK3 w - - 0 1").unwrap();

    assert!(evaluate(&white_extra_queen) > evaluate(&balanced));
}

#[test]
#[ignore = "benchmark-style test"]
fn benchmark_nodes_per_second_fixed_budget() {
    let state =
        parse_fen("r1bq1rk1/pp2bppp/2n1pn2/2pp4/3P4/2PBPN2/PP3PPP/RNBQ1RK1 w - - 0 9").unwrap();
    let mut searcher = Searcher::new(SearchConfig {
        max_depth: 64,
        time_limit_ms: Some(500),
    });

    let result = searcher.search(&state);
    let nps = result.nodes as f64 / 0.5;

    println!(
        "nodes={} depth={} nps={:.0}",
        result.nodes, result.depth, nps
    );
    assert!(result.nodes > 0);
    assert!(result.depth > 0);
}

#[test]
#[ignore = "benchmark-style test"]
fn benchmark_depth_reached_by_time_budget() {
    let state =
        parse_fen("rnbq1rk1/pp3ppp/2pbpn2/3p4/3P4/2N1PN2/PPQ1BPPP/R1B2RK1 w - - 0 8").unwrap();

    let budgets = [100_u64, 250_u64, 500_u64];
    let mut previous_depth = 0;
    for budget in budgets {
        let mut searcher = Searcher::new(SearchConfig {
            max_depth: 64,
            time_limit_ms: Some(budget),
        });
        let result = searcher.search(&state);
        println!(
            "budget_ms={} depth={} nodes={}",
            budget, result.depth, result.nodes
        );

        assert!(result.depth >= previous_depth);
        previous_depth = result.depth;
    }
}
