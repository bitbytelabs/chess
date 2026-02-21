use std::env;

use chess::search::searcher::SearchConfig;
use chess::uci::UciEngine;

fn main() {
    let mut depth = 4u8;
    let mut movetime_ms = None;

    let args: Vec<String> = env::args().collect();
    let mut i = 1usize;
    while i < args.len() {
        match args[i].as_str() {
            "--depth" if i + 1 < args.len() => {
                if let Ok(d) = args[i + 1].parse::<u8>() {
                    depth = d;
                }
                i += 1;
            }
            "--movetime-ms" if i + 1 < args.len() => {
                if let Ok(ms) = args[i + 1].parse::<u64>() {
                    movetime_ms = Some(ms);
                }
                i += 1;
            }
            "--help" | "-h" => {
                println!("Usage: chess [--depth N] [--movetime-ms MS]");
                return;
            }
            _ => {}
        }
        i += 1;
    }

    let config = SearchConfig {
        max_depth: depth,
        time_limit_ms: movetime_ms,
    };

    let mut engine = UciEngine::new(config);
    engine.run();
}
