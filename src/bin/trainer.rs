use std::env;
use std::io::{self, BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

use chess::engine::board::{Color, Move, Piece, PieceType};
use chess::engine::fen::{parse_fen, STARTPOS_FEN};
use chess::engine::game_state::GameState;
use chess::engine::legal::{generate_legal_moves, in_check};

#[derive(Clone, Debug)]
struct Config {
    candidate: String,
    baseline: String,
    games: u32,
    movetime_ms: u64,
    max_plies: u32,
    opening_random_plies: u32,
    seed: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            candidate: String::new(),
            baseline: String::new(),
            games: 20,
            movetime_ms: 100,
            max_plies: 200,
            opening_random_plies: 6,
            seed: 42,
        }
    }
}

#[derive(Default, Debug)]
struct Score {
    wins: u32,
    draws: u32,
    losses: u32,
}

impl Score {
    fn games(&self) -> u32 {
        self.wins + self.draws + self.losses
    }

    fn score_rate(&self) -> f64 {
        if self.games() == 0 {
            return 0.5;
        }
        (f64::from(self.wins) + 0.5 * f64::from(self.draws)) / f64::from(self.games())
    }

    fn elo_estimate(&self) -> f64 {
        elo_from_score(self.score_rate())
    }

    fn record(&mut self, points: f64) {
        if (points - 1.0).abs() < f64::EPSILON {
            self.wins += 1;
        } else if (points - 0.5).abs() < f64::EPSILON {
            self.draws += 1;
        } else {
            self.losses += 1;
        }
    }
}

struct SmallRng {
    state: u64,
}

impl SmallRng {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.state
    }

    fn pick_index(&mut self, len: usize) -> usize {
        (self.next_u64() % len as u64) as usize
    }
}

struct EngineProcess {
    _child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

impl EngineProcess {
    fn launch(path: &str) -> Result<Self, String> {
        let mut child = Command::new(path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .map_err(|e| format!("failed to start engine '{path}': {e}"))?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| format!("failed to open stdin for '{path}'"))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| format!("failed to open stdout for '{path}'"))?;

        let mut engine = Self {
            _child: child,
            stdin,
            stdout: BufReader::new(stdout),
        };

        engine.send_line("uci")?;
        engine.read_until_prefix("uciok")?;
        engine.send_line("isready")?;
        engine.read_until_prefix("readyok")?;
        Ok(engine)
    }

    fn send_line(&mut self, line: &str) -> Result<(), String> {
        writeln!(self.stdin, "{line}").map_err(io_err)?;
        self.stdin.flush().map_err(io_err)
    }

    fn read_until_prefix(&mut self, prefix: &str) -> Result<String, String> {
        let mut buf = String::new();
        loop {
            buf.clear();
            let n = self.stdout.read_line(&mut buf).map_err(io_err)?;
            if n == 0 {
                return Err("engine closed stdout unexpectedly".to_owned());
            }
            let line = buf.trim();
            if line.starts_with(prefix) {
                return Ok(line.to_owned());
            }
        }
    }

    fn bestmove(&mut self, state: &GameState, movetime_ms: u64) -> Result<String, String> {
        let fen = game_state_to_fen(state);
        self.send_line(&format!("position fen {fen}"))?;
        self.send_line(&format!("go movetime {movetime_ms}"))?;
        let line = self.read_until_prefix("bestmove")?;
        let mut it = line.split_whitespace();
        let _ = it.next();
        let mv = it
            .next()
            .ok_or_else(|| format!("malformed bestmove line: {line}"))?;
        Ok(mv.to_owned())
    }
}

#[derive(Copy, Clone)]
enum GameResult {
    WhiteWin,
    BlackWin,
    Draw,
}

fn elo_from_score(score: f64) -> f64 {
    let p = score.clamp(1e-6, 1.0 - 1e-6);
    -400.0 * ((1.0 / p) - 1.0).log10()
}

fn piece_to_fen_char(piece: Piece) -> char {
    let ch = match piece.kind {
        PieceType::Pawn => 'p',
        PieceType::Knight => 'n',
        PieceType::Bishop => 'b',
        PieceType::Rook => 'r',
        PieceType::Queen => 'q',
        PieceType::King => 'k',
    };
    if piece.color == Color::White {
        ch.to_ascii_uppercase()
    } else {
        ch
    }
}

fn game_state_to_fen(state: &GameState) -> String {
    let mut board = String::new();
    for rank in (0..8).rev() {
        let mut empty = 0u8;
        for file in 0..8 {
            let sq = rank * 8 + file;
            match state.board[sq] {
                Some(piece) => {
                    if empty > 0 {
                        board.push(char::from(b'0' + empty));
                        empty = 0;
                    }
                    board.push(piece_to_fen_char(piece));
                }
                None => empty += 1,
            }
        }
        if empty > 0 {
            board.push(char::from(b'0' + empty));
        }
        if rank > 0 {
            board.push('/');
        }
    }

    let side = if state.side_to_move == Color::White {
        "w"
    } else {
        "b"
    };

    let mut castling = String::new();
    if state.castling.white_king_side {
        castling.push('K');
    }
    if state.castling.white_queen_side {
        castling.push('Q');
    }
    if state.castling.black_king_side {
        castling.push('k');
    }
    if state.castling.black_queen_side {
        castling.push('q');
    }
    if castling.is_empty() {
        castling.push('-');
    }

    let ep = state
        .en_passant
        .map(square_to_text)
        .unwrap_or_else(|| "-".to_owned());

    format!(
        "{} {} {} {} {} {}",
        board, side, castling, ep, state.halfmove_clock, state.fullmove_number
    )
}

fn square_to_text(sq: usize) -> String {
    let file = (b'a' + (sq % 8) as u8) as char;
    let rank = (b'1' + (sq / 8) as u8) as char;
    format!("{file}{rank}")
}

fn parse_config() -> Result<Config, String> {
    let mut cfg = Config::default();
    let args: Vec<String> = env::args().collect();

    let mut i = 1usize;
    while i < args.len() {
        match args[i].as_str() {
            "--candidate" if i + 1 < args.len() => {
                cfg.candidate = args[i + 1].clone();
                i += 1;
            }
            "--baseline" if i + 1 < args.len() => {
                cfg.baseline = args[i + 1].clone();
                i += 1;
            }
            "--games" if i + 1 < args.len() => {
                cfg.games = args[i + 1]
                    .parse()
                    .map_err(|_| "--games must be a positive integer".to_owned())?;
                i += 1;
            }
            "--movetime-ms" if i + 1 < args.len() => {
                cfg.movetime_ms = args[i + 1]
                    .parse()
                    .map_err(|_| "--movetime-ms must be a positive integer".to_owned())?;
                i += 1;
            }
            "--max-plies" if i + 1 < args.len() => {
                cfg.max_plies = args[i + 1]
                    .parse()
                    .map_err(|_| "--max-plies must be a positive integer".to_owned())?;
                i += 1;
            }
            "--opening-random-plies" if i + 1 < args.len() => {
                cfg.opening_random_plies = args[i + 1]
                    .parse()
                    .map_err(|_| "--opening-random-plies must be a positive integer".to_owned())?;
                i += 1;
            }
            "--seed" if i + 1 < args.len() => {
                cfg.seed = args[i + 1]
                    .parse()
                    .map_err(|_| "--seed must be an integer".to_owned())?;
                i += 1;
            }
            "--help" | "-h" => {
                print_usage();
                std::process::exit(0);
            }
            other => {
                return Err(format!("unknown or incomplete argument: {other}"));
            }
        }
        i += 1;
    }

    if cfg.candidate.is_empty() || cfg.baseline.is_empty() {
        return Err("--candidate and --baseline are required".to_owned());
    }

    Ok(cfg)
}

fn print_usage() {
    println!(
        "Usage: trainer --candidate <path> --baseline <path> [--games N] [--movetime-ms MS] [--max-plies N] [--opening-random-plies N] [--seed N]"
    );
}

fn randomize_opening(state: &mut GameState, rng: &mut SmallRng, opening_plies: u32) {
    for _ in 0..opening_plies {
        let legal = generate_legal_moves(state);
        if legal.is_empty() {
            break;
        }
        let pick = rng.pick_index(legal.len());
        state.make_move(legal[pick]);
    }
}

fn pick_legal_move_from_uci(state: &GameState, uci: &str) -> Option<Move> {
    generate_legal_moves(state)
        .into_iter()
        .find(|mv| mv.to_uci() == uci)
}

fn play_one_game(
    candidate: &mut EngineProcess,
    baseline: &mut EngineProcess,
    candidate_is_white: bool,
    cfg: &Config,
    rng: &mut SmallRng,
) -> Result<GameResult, String> {
    let mut state =
        parse_fen(STARTPOS_FEN).map_err(|e| format!("failed to parse start FEN: {e}"))?;
    randomize_opening(&mut state, rng, cfg.opening_random_plies);

    for _ in 0..cfg.max_plies {
        let legal = generate_legal_moves(&state);
        if legal.is_empty() {
            let result = if in_check(&state, state.side_to_move) {
                if state.side_to_move == Color::White {
                    GameResult::BlackWin
                } else {
                    GameResult::WhiteWin
                }
            } else {
                GameResult::Draw
            };
            return Ok(result);
        }

        let candidate_turn = (state.side_to_move == Color::White && candidate_is_white)
            || (state.side_to_move == Color::Black && !candidate_is_white);

        let raw_move = if candidate_turn {
            candidate.bestmove(&state, cfg.movetime_ms)?
        } else {
            baseline.bestmove(&state, cfg.movetime_ms)?
        };

        let parsed_move = pick_legal_move_from_uci(&state, &raw_move)
            .ok_or_else(|| format!("engine returned illegal move: {raw_move}"))?;
        state.make_move(parsed_move);
    }

    Ok(GameResult::Draw)
}

fn game_points_for_candidate(result: GameResult, candidate_is_white: bool) -> f64 {
    match result {
        GameResult::Draw => 0.5,
        GameResult::WhiteWin => {
            if candidate_is_white {
                1.0
            } else {
                0.0
            }
        }
        GameResult::BlackWin => {
            if candidate_is_white {
                0.0
            } else {
                1.0
            }
        }
    }
}

fn io_err(err: io::Error) -> String {
    err.to_string()
}

fn run() -> Result<(), String> {
    let cfg = parse_config()?;

    let mut rng = SmallRng::new(cfg.seed);
    let mut score = Score::default();
    let mut candidate_engine = EngineProcess::launch(&cfg.candidate)?;
    let mut baseline_engine = EngineProcess::launch(&cfg.baseline)?;

    for game_idx in 0..cfg.games {
        let candidate_is_white = game_idx % 2 == 0;
        let result = play_one_game(
            &mut candidate_engine,
            &mut baseline_engine,
            candidate_is_white,
            &cfg,
            &mut rng,
        )?;

        let points = game_points_for_candidate(result, candidate_is_white);
        score.record(points);
        println!(
            "game {}/{}: candidate_is_white={} score={} W-D-L={}-{}-{} elo_est={:.1}",
            game_idx + 1,
            cfg.games,
            candidate_is_white,
            points,
            score.wins,
            score.draws,
            score.losses,
            score.elo_estimate()
        );
    }

    println!("\n=== Summary ===");
    println!(
        "Candidate W-D-L: {}-{}-{}",
        score.wins, score.draws, score.losses
    );
    println!("Score rate: {:.3}", score.score_rate());
    println!("Estimated Elo delta: {:.1}", score.elo_estimate());

    Ok(())
}

fn main() {
    if let Err(err) = run() {
        eprintln!("trainer error: {err}");
        print_usage();
        std::process::exit(2);
    }
}

#[cfg(test)]
mod tests {
    use super::{elo_from_score, game_state_to_fen};
    use chess::engine::fen::{parse_fen, STARTPOS_FEN};

    #[test]
    fn elo_estimate_is_zero_for_50_percent_score() {
        let elo = elo_from_score(0.5);
        assert!(elo.abs() < 1e-9);
    }

    #[test]
    fn fen_serialization_round_trip_startpos() {
        let state = parse_fen(STARTPOS_FEN).expect("valid start fen");
        let fen = game_state_to_fen(&state);
        let parsed = parse_fen(&fen).expect("round trip fen parses");
        assert_eq!(state.board, parsed.board);
        assert_eq!(state.side_to_move, parsed.side_to_move);
        assert_eq!(state.castling, parsed.castling);
        assert_eq!(state.en_passant, parsed.en_passant);
    }
}
