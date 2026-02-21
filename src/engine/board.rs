use std::fmt;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Color {
    White,
    Black,
}

impl Color {
    pub fn opposite(self) -> Self {
        match self {
            Self::White => Self::Black,
            Self::Black => Self::White,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum PieceType {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Piece {
    pub color: Color,
    pub kind: PieceType,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Move {
    pub from: usize,
    pub to: usize,
    pub promotion: Option<PieceType>,
    pub is_en_passant: bool,
    pub is_castling: bool,
}

impl Move {
    pub fn new(from: usize, to: usize) -> Self {
        Self {
            from,
            to,
            promotion: None,
            is_en_passant: false,
            is_castling: false,
        }
    }

    pub fn to_uci(self) -> String {
        let mut text = format!("{}{}", square_to_str(self.from), square_to_str(self.to));
        if let Some(p) = self.promotion {
            text.push(match p {
                PieceType::Knight => 'n',
                PieceType::Bishop => 'b',
                PieceType::Rook => 'r',
                PieceType::Queen => 'q',
                _ => 'q',
            });
        }
        text
    }
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_uci())
    }
}

pub fn square(file: usize, rank: usize) -> usize {
    rank * 8 + file
}

pub fn file_of(sq: usize) -> usize {
    sq % 8
}

pub fn rank_of(sq: usize) -> usize {
    sq / 8
}

pub fn square_to_str(sq: usize) -> String {
    let file = (b'a' + file_of(sq) as u8) as char;
    let rank = (b'1' + rank_of(sq) as u8) as char;
    format!("{file}{rank}")
}

pub fn str_to_square(input: &str) -> Option<usize> {
    if input.len() != 2 {
        return None;
    }
    let bytes = input.as_bytes();
    if !(b'a'..=b'h').contains(&bytes[0]) || !(b'1'..=b'8').contains(&bytes[1]) {
        return None;
    }
    Some(square(
        (bytes[0] - b'a') as usize,
        (bytes[1] - b'1') as usize,
    ))
}
