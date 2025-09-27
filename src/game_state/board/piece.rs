//! # Piece Definitions Module
//!
//! This module defines the core chess piece types, colors, and related
//! utility methods used throughout the engine.
//!
//! It provides:
//! - The [`PieceType`] enum, representing the abstract kind of a piece (king,
//!   queen, rook, bishop, knight, pawn).
//! - The [`Color`] enum, representing the side to which a piece belongs
//!   (white or black), including helper methods like [`Color::opposite`].
//! - The [`Piece`] enum, representing the concrete pieces (e.g. `WhitePawn`,
//!   `BlackQueen`) plus sentinel and empty squares, with methods for
//!   color/type queries, printing symbols, and checking relationships
//!   (friend, opponent, empty, sentinel).
//!
//! In short, this module is the foundation for representing pieces on the
//! board and is used by move generation, evaluation, and other parts of the
//! chess engine.

/// Represents the type of a chess piece, without its color.
///
/// Used to differentiate between different movement patterns
/// (e.g., `PieceType::Rook` vs `PieceType::Bishop`).
#[derive(PartialEq)]
pub enum PieceType {
    King,
    Queen,
    Rook,
    Bishop,
    Knight,
    Pawn,
}

/// Represents the color of a piece or side to move.
#[derive(PartialEq, Copy, Clone, Debug)]
pub enum Color {
    /// White side
    White,
    /// Black side
    Black,
}

impl Color {
    /// Returns the opposite color.
    ///
    /// # Examples
    /// ```
    /// use enrust::game_state::Color;
    /// assert_eq!(Color::White.opposite(), Color::Black);
    /// ```
    pub fn opposite(&self) -> Color {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

/// Represents an actual piece on the board, including its color.
///
/// This also includes two special values:
/// * `EmptySquare` for empty squares
/// * `SentinelSquare` for off-board sentinel squares (used by 12×10 boards)
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Piece {
    /// Empty square (no piece present)
    EmptySquare = 0,
    WhitePawn,
    WhiteKnight,
    WhiteBishop,
    WhiteRook,
    WhiteQueen,
    WhiteKing,
    BlackPawn,
    BlackKnight,
    BlackBishop,
    BlackRook,
    BlackQueen,
    BlackKing,
    /// Sentinel square (off-board guard)
    SentinelSquare = 255,
}

impl Piece {
    /// Returns the color of the piece.
    ///
    /// # Panics
    /// Panics if called on an empty or sentinel square.
    pub fn get_color(self) -> Color {
        match self as u8 {
            1..=6 => Color::White,
            7..=12 => Color::Black,
            _ => panic!("Invalid piece"),
        }
    }

    /// Returns the type of the piece (king, queen, etc.).
    ///
    /// # Panics
    /// Panics if called on an empty or sentinel square.
    pub fn get_type(self) -> PieceType {
        match self {
            Piece::WhitePawn | Piece::BlackPawn => PieceType::Pawn,
            Piece::WhiteKnight | Piece::BlackKnight => PieceType::Knight,
            Piece::WhiteBishop | Piece::BlackBishop => PieceType::Bishop,
            Piece::WhiteRook | Piece::BlackRook => PieceType::Rook,
            Piece::WhiteQueen | Piece::BlackQueen => PieceType::Queen,
            Piece::WhiteKing | Piece::BlackKing => PieceType::King,
            _ => panic!("Invalid piece"),
        }
    }

    /// Returns the single-character string used to print the piece on a board.
    ///
    /// Empty squares are `"."`, sentinel squares are `"X"`.
    pub fn print_piece(&self) -> &str {
        match self {
            Piece::EmptySquare => ".",
            Piece::SentinelSquare => "X",
            Piece::WhitePawn => "P",
            Piece::WhiteRook => "R",
            Piece::WhiteKnight => "N",
            Piece::WhiteBishop => "B",
            Piece::WhiteQueen => "Q",
            Piece::WhiteKing => "K",
            Piece::BlackPawn => "p",
            Piece::BlackRook => "r",
            Piece::BlackKnight => "n",
            Piece::BlackBishop => "b",
            Piece::BlackQueen => "q",
            Piece::BlackKing => "k",
        }
    }

    /// Returns `true` if the square is empty.
    pub fn is_empty(self) -> bool {
        self == Piece::EmptySquare
    }

    /// Returns `true` if the square is a sentinel square.
    pub fn is_sentinel(self) -> bool {
        self == Piece::SentinelSquare
    }

    /// Returns `true` if the piece is one of the 12 valid chess pieces.
    pub fn is_valid_piece(self) -> bool {
        (self as u8) >= 1 && (self as u8) <= 12
    }

    /// Returns `true` if the piece is white.
    pub fn is_white(self) -> bool {
        self.is_color(Color::White)
    }

    /// Internal helper to check a specific color.
    fn is_color(self, color: Color) -> bool {
        if !self.is_valid_piece() {
            return false;
        }
        self.get_color() == color
    }

    /// Returns `true` if the piece belongs to the opponent of `color`.
    pub fn is_opponent(self, color: Color) -> bool {
        self.is_valid_piece() && self.get_color() != color
    }

    /// Returns `true` if the piece belongs to the same side as `color`.
    pub fn is_friend(self, color: Color) -> bool {
        self.is_valid_piece() && self.get_color() == color
    }
}
