use std::str::FromStr;

use crate::engine::board::{square, Color, Piece, PieceType};
use crate::engine::game_state::{CastlingRights, GameState};

pub const STARTPOS_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

pub fn parse_fen(fen: &str) -> Result<GameState, String> {
    let parts: Vec<&str> = fen.split_whitespace().collect();
    if parts.len() != 6 {
        return Err("FEN must have 6 fields".to_owned());
    }

    let mut state = GameState::default();
    parse_board(parts[0], &mut state)?;
    state.side_to_move = match parts[1] {
        "w" => Color::White,
        "b" => Color::Black,
        _ => return Err("Invalid side to move".to_owned()),
    };
    state.castling = parse_castling(parts[2]);
    state.en_passant = if parts[3] == "-" {
        None
    } else {
        Some(parse_square(parts[3])?)
    };
    state.halfmove_clock =
        u32::from_str(parts[4]).map_err(|_| "Invalid halfmove clock".to_owned())?;
    state.fullmove_number =
        u32::from_str(parts[5]).map_err(|_| "Invalid fullmove number".to_owned())?;

    Ok(state)
}

fn parse_board(data: &str, state: &mut GameState) -> Result<(), String> {
    let ranks: Vec<&str> = data.split('/').collect();
    if ranks.len() != 8 {
        return Err("Board field must have 8 ranks".to_owned());
    }

    for (i, rank_data) in ranks.iter().enumerate() {
        let board_rank = 7 - i;
        let mut file = 0usize;
        for ch in rank_data.chars() {
            if ch.is_ascii_digit() {
                file += ch.to_digit(10).unwrap() as usize;
                continue;
            }
            let piece =
                piece_from_char(ch).ok_or_else(|| format!("Invalid piece character: {ch}"))?;
            if file >= 8 {
                return Err("Too many files in rank".to_owned());
            }
            state.board[square(file, board_rank)] = Some(piece);
            file += 1;
        }
        if file != 8 {
            return Err("Rank does not have exactly 8 squares".to_owned());
        }
    }

    Ok(())
}

fn piece_from_char(ch: char) -> Option<Piece> {
    let color = if ch.is_uppercase() {
        Color::White
    } else {
        Color::Black
    };
    let kind = match ch.to_ascii_lowercase() {
        'p' => PieceType::Pawn,
        'n' => PieceType::Knight,
        'b' => PieceType::Bishop,
        'r' => PieceType::Rook,
        'q' => PieceType::Queen,
        'k' => PieceType::King,
        _ => return None,
    };
    Some(Piece { color, kind })
}

fn parse_castling(input: &str) -> CastlingRights {
    let mut rights = CastlingRights {
        white_king_side: false,
        white_queen_side: false,
        black_king_side: false,
        black_queen_side: false,
    };

    if input.contains('K') {
        rights.white_king_side = true;
    }
    if input.contains('Q') {
        rights.white_queen_side = true;
    }
    if input.contains('k') {
        rights.black_king_side = true;
    }
    if input.contains('q') {
        rights.black_queen_side = true;
    }

    rights
}

fn parse_square(input: &str) -> Result<usize, String> {
    if input.len() != 2 {
        return Err("Invalid square format".to_owned());
    }
    let bytes = input.as_bytes();
    if !(b'a'..=b'h').contains(&bytes[0]) || !(b'1'..=b'8').contains(&bytes[1]) {
        return Err("Square out of range".to_owned());
    }
    Ok(square(
        (bytes[0] - b'a') as usize,
        (bytes[1] - b'1') as usize,
    ))
}
