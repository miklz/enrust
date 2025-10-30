//! Chess board representation and game state management.
//!
//! This module provides the core data structures and logic for representing
//! a chess board, managing piece positions, handling moves, and evaluating
//! game states. The board uses a 12x10 mailbox representation with sentinel
//! squares for efficient move generation and validation.

use std::sync::Arc;
use std::sync::atomic::AtomicBool;

pub mod moves;
pub mod piece;
pub mod piece_list;
pub mod search;
pub mod transposition_table;

use moves::Move;
use piece::{Color, Piece, PieceType};
use piece_list::PieceList;
use transposition_table::{TranspositionTable, Zobrist};

/// Represents the castling rights for both players.
///
/// Tracks which castling moves are still available for white and black,
/// both kingside and queenside. Castling rights are updated automatically
/// when pieces move or are captured.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct CastlingRights {
    /// Whether white can still castle queenside
    pub white_queenside: bool,
    /// Whether white can still castle kingside
    pub white_kingside: bool,
    /// Whether black can still castle queenside
    pub black_queenside: bool,
    /// Whether black can still castle kingside
    pub black_kingside: bool,
}

/// Contains information needed to execute a castling move.
///
/// Stores the rook's movement details for castling operations.
#[derive(Clone, Debug, PartialEq)]
pub struct CastlingInfo {
    /// The rook's starting square
    pub rook_from: i16,
    /// The rook's destination square after castling
    pub rook_to: i16,
    /// The rook piece being moved
    pub rook_piece: Piece,
}

/// Main chess board representation using a mailbox system.
///
/// The board uses a 12x10 array with sentinel squares around the edges
/// to simplify boundary checking during move generation. The actual chess
/// board occupies the central 8x8 area.
#[derive(Clone)]
pub struct ChessBoard {
    /// Width of the internal board representation (including sentinels)
    board_width: i16,
    /// Height of the internal board representation (including sentinels)
    board_height: i16,
    /// Array of pieces representing the board state with sentinel borders
    board_squares: [Piece; 12 * 10],

    /// The en passant target square, if applicable
    en_passant_target: Option<i16>,

    /// Current castling rights for both players
    castling_rights: CastlingRights,

    /// Piece lists for efficient piece tracking and move generation
    piece_list: PieceList,

    /// Zobrist structure with random numbers
    zobrist: Arc<Zobrist>,

    /// Hash value that represents this board position
    hash: u64,

    /// Transposition Table
    transposition_table: Arc<TranspositionTable>,
}

impl ChessBoard {
    /// Calculates the material score for the current board position.
    ///
    /// Uses standard chess piece values:
    /// - King: 20000
    /// - Queen: 900
    /// - Rook: 500
    /// - Bishop/Knight: 300
    /// - Pawn: 100
    ///
    /// # Arguments
    ///
    /// * `piece_list` - Reference to the piece list to evaluate
    ///
    /// # Returns
    ///
    /// Material score from white's perspective (positive if white has advantage)
    fn material_score(&self, piece_list: &PieceList) -> i16 {
        // count pieces
        let w_king = piece_list
            .get_number_of_pieces(Piece::WhiteKing)
            .unwrap_or(0);
        let b_king = piece_list
            .get_number_of_pieces(Piece::BlackKing)
            .unwrap_or(0);
        let w_queen = piece_list
            .get_number_of_pieces(Piece::WhiteQueen)
            .unwrap_or(0);
        let b_queen = piece_list
            .get_number_of_pieces(Piece::BlackQueen)
            .unwrap_or(0);
        let w_rook = piece_list
            .get_number_of_pieces(Piece::WhiteRook)
            .unwrap_or(0);
        let b_rook = piece_list
            .get_number_of_pieces(Piece::BlackRook)
            .unwrap_or(0);
        let w_bishop = piece_list
            .get_number_of_pieces(Piece::WhiteBishop)
            .unwrap_or(0);
        let b_bishop = piece_list
            .get_number_of_pieces(Piece::BlackBishop)
            .unwrap_or(0);
        let w_knight = piece_list
            .get_number_of_pieces(Piece::WhiteKnight)
            .unwrap_or(0);
        let b_kinght = piece_list
            .get_number_of_pieces(Piece::BlackKnight)
            .unwrap_or(0);
        let w_pawn = piece_list
            .get_number_of_pieces(Piece::WhitePawn)
            .unwrap_or(0);
        let b_pawn = piece_list
            .get_number_of_pieces(Piece::BlackPawn)
            .unwrap_or(0);

        20000 * (w_king - b_king)
            + 900 * (w_queen - b_queen)
            + 500 * (w_rook - b_rook)
            + 300 * (w_bishop - b_bishop + w_knight - b_kinght)
            + 100 * (w_pawn - b_pawn)
    }

    /// Evaluates the current board position using material counting.
    ///
    /// # Returns
    ///
    /// Score from white's perspective (positive if white is winning)
    pub fn evaluate(&self) -> i16 {
        self.material_score(&self.piece_list)
    }

    /// Checks if the given color is in checkmate.
    ///
    /// # Arguments
    ///
    /// * `color` - Color to check for checkmate
    ///
    /// # Returns
    ///
    /// `true` if the king is in check and no legal moves exist
    pub fn is_checkmate(&mut self, color: Color) -> bool {
        let moves = self.generate_moves(color);
        moves.is_empty() && self.is_in_check(color)
    }

    /// Checks if the given color's king is in check.
    ///
    /// # Arguments
    ///
    /// * `color` - Color to check for check
    ///
    /// # Returns
    ///
    /// `true` if the king is under attack
    pub fn is_in_check(&self, color: Color) -> bool {
        !self.piece_list.is_king_in_check(self, color).is_empty()
    }

    /// Parses a move from UCI algebraic notation.
    ///
    /// # Arguments
    ///
    /// * `uci_notation` - Move in UCI format (e.g., "e2e4", "g1f3")
    ///
    /// # Returns
    ///
    /// `Some(Move)` if the notation is valid, `None` otherwise
    pub fn from_uci(&self, uci_notation: &str) -> Option<Move> {
        Move::parse_algebraic_move(self, uci_notation)
    }

    /// Converts a move to UCI algebraic notation.
    ///
    /// # Arguments
    ///
    /// * `mv` - The move to convert
    ///
    /// # Returns
    ///
    /// UCI string representation of the move
    pub fn move_to_uci(&self, mv: &Move) -> String {
        mv.to_uci(self)
    }

    /// Converts algebraic notation to internal board coordinates.
    ///
    /// # Arguments
    ///
    /// * `algebraic_notation` - Square in algebraic notation (e.g., "e4")
    ///
    /// # Returns
    ///
    /// Internal board index, or -1 if invalid
    fn algebraic_to_internal(&self, algebraic_notation: &str) -> i16 {
        if let Some(square) = Move::notation_to_square(algebraic_notation) {
            return self.map_inner_to_outer_board(square);
        }
        -1
    }

    /// Gets the piece on a given square.
    ///
    /// # Arguments
    ///
    /// * `square` - Internal board coordinate
    ///
    /// # Returns
    ///
    /// Piece at the specified square
    fn get_piece_on_square(&self, square: i16) -> Piece {
        self.board_squares[square as usize]
    }

    /// Sets a piece on a given square.
    ///
    /// # Arguments
    ///
    /// * `piece` - Piece to place
    /// * `square` - Internal board coordinate
    fn set_piece_on_square(&mut self, piece: Piece, square: i16) {
        self.board_squares[square as usize] = piece;
    }

    /// Checks if two squares are on the same rank (row).
    ///
    /// # Arguments
    ///
    /// * `square1` - First square to compare
    /// * `square2` - Second square to compare
    ///
    /// # Returns
    ///
    /// `true` if both squares are on the same rank
    fn are_on_the_same_rank(&self, square1: i16, square2: i16) -> bool {
        // Two squares are on the same rank (row) if their indices divided by board_width are equal.
        square1 / self.board_width == square2 / self.board_width
    }

    /// Checks if two squares are on the same file (column).
    ///
    /// # Arguments
    ///
    /// * `square1` - First square to compare
    /// * `square2` - Second square to compare
    ///
    /// # Returns
    ///
    /// `true` if both squares are on the same file
    fn are_on_the_same_file(&self, square1: i16, square2: i16) -> bool {
        // Two squares are on the same file (column) if their indices modulo board_width are equal.
        square1 % self.board_width == square2 % self.board_width
    }

    /// Checks if two squares are on the same diagonal.
    ///
    /// # Arguments
    ///
    /// * `square1` - First square to compare
    /// * `square2` - Second square to compare
    ///
    /// # Returns
    ///
    /// `true` if both squares are on the same diagonal
    fn are_on_the_same_diagonal(&self, square1: i16, square2: i16) -> bool {
        let row1 = square1 / self.board_width;
        let col1 = square1 % self.board_width;

        let row2 = square2 / self.board_width;
        let col2 = square2 % self.board_width;

        // Squares are on the same diagonal if the absolute difference in rows
        // equals the absolute difference in columns
        row1.abs_diff(row2) == col1.abs_diff(col2)
    }

    /// Gets all squares between two positions (exclusive).
    ///
    /// Only works for straight lines (ranks, files) or diagonals.
    ///
    /// # Arguments
    ///
    /// * `from` - Starting square
    /// * `to` - Ending square
    ///
    /// # Returns
    ///
    /// Vector of squares between the two positions
    fn get_squares_between(&self, from: i16, to: i16) -> Vec<i16> {
        let mut squares = Vec::new();

        let from_rank = self.square_rank(from);
        let from_file = self.square_file(from);
        let to_rank = self.square_rank(to);
        let to_file = self.square_file(to);

        let rank_diff = to_rank - from_rank;
        let file_diff = to_file - from_file;

        // Only straight or diagonal lines have squares between them
        if rank_diff == 0 || file_diff == 0 || rank_diff.abs() == file_diff.abs() {
            let rank_step = rank_diff.signum();
            let file_step = file_diff.signum();
            let steps = rank_diff.abs().max(file_diff.abs());

            for i in 1..steps {
                let rank = from_rank + i * rank_step;
                let file = from_file + i * file_step;
                squares.push(rank * self.board_width + file);
            }
        }

        squares
    }

    /// Get direction that a square can reach another in straight lines.
    ///
    /// Only works for straight lines (ranks, files)
    ///
    /// # Arguments
    ///
    /// * `from` - Starting square
    /// * `to` - Ending square
    ///
    /// # Returns
    ///
    /// Direction that should be taked to reach end square or 0 if there's not
    /// a valid straight line between `from` and `to` squares
    fn get_rank_or_file_direction(&self, from: i16, to: i16) -> i16 {
        // Sanity check, the squares can't be the same
        if from == to {
            return 0;
        }

        // Check if the squares are in the same file or in the same rank.
        let same_rank = self.are_on_the_same_rank(from, to);
        let same_file = self.are_on_the_same_file(from, to);
        if !same_file && !same_rank {
            // If they aren't in the same rank or in the same file,
            // the rook can't move there.
            return 0;
        }

        // We now know that the squares are in the same file or in the
        // same rank, we need to get in which direction the rook should
        // move.
        let distance = to - from;
        if same_rank {
            if distance > 0 { 1 } else { -1 }
        } else if distance > 0 {
            self.board_width
        } else {
            -self.board_width
        }
    }

    /// Get the direction that if a square can reach another in diagonal lines.
    ///
    /// Only works for diagonal lines
    ///
    /// # Arguments
    ///
    /// * `from` - Starting square
    /// * `to` - Ending square
    ///
    /// # Returns
    ///
    /// Direction it should be taked to reach end square or 0 if there's not
    /// a valid diagonal line between `from` and `to` squares
    fn get_diagonal_direction(&self, from: i16, to: i16) -> i16 {
        // Sanity check, the squares can't be the same
        if from == to {
            return 0;
        }

        // Check if the squares are in the same diagonal.
        let same_diagonal = self.are_on_the_same_diagonal(from, to);
        if !same_diagonal {
            // If they aren't in the same diagonal the bishop can't move there
            return 0;
        }

        // The squares are in the same diagonal, now we need to get in which
        // direction the bishop should move.
        let row1 = self.square_rank(from);
        let row2 = self.square_rank(to);
        let row_dir: i16 = if row2 > row1 { 1 } else { -1 };

        let col1 = self.square_file(from);
        let col2 = self.square_file(to);
        let col_dir: i16 = if col2 > col1 { 1 } else { -1 };

        row_dir * self.board_width + col_dir
    }

    /// Gets the current en passant target square.
    ///
    /// # Returns
    ///
    /// `Some(square)` if en passant is possible, `None` otherwise
    fn get_en_passant_target(&self) -> Option<i16> {
        self.en_passant_target
    }

    /// Sets the en passant target square.
    ///
    /// # Arguments
    ///
    /// * `square` - New en passant target square
    fn set_en_passant_target(&mut self, square: Option<i16>) {
        self.en_passant_target = square;
    }

    /// Gets the rank (row) of a square.
    ///
    /// # Arguments
    ///
    /// * `square` - Internal board coordinate
    ///
    /// # Returns
    ///
    /// Rank index (0-7) within the chess board
    fn square_rank(&self, square: i16) -> i16 {
        square / self.board_width
    }

    /// Gets the file (column) of a square.
    ///
    /// # Arguments
    ///
    /// * `square` - Internal board coordinate
    ///
    /// # Returns
    ///
    /// File index (0-7) within the chess board
    fn square_file(&self, square: i16) -> i16 {
        square % self.board_width
    }

    /// Maps a standard chess square (0-63) to internal board coordinates.
    ///
    /// The internal board uses a 12x10 mailbox representation with sentinel squares.
    ///
    /// # Arguments
    ///
    /// * `square` - Standard chess square index (0-63)
    ///
    /// # Returns
    ///
    /// Internal board coordinate
    fn map_inner_to_outer_board(&self, square: i16) -> i16 {
        // We have a larger board with sentinel squares around the edges.
        // This function converts a standard 0-63 chess square to its position
        // in our internal board representation.

        // Calculate the starting position of the inner 8×8 board within our larger board
        let vertical_padding = (self.board_height - 8) / 2; // Rows below the chess board
        let horizontal_padding = (self.board_width - 8) / 2; // Columns to the left

        let board_offset = vertical_padding * self.board_width + horizontal_padding;

        // Convert standard chess coordinates to internal board coordinates
        let chess_rank = square / 8;
        let chess_file = square % 8;

        // Internal position = (rows above) + (chess rank) × (board width) + (columns left) + (chess file)

        self.board_width * chess_rank + chess_file + board_offset
    }

    /// Maps an internal board coordinate to standard chess square index.
    ///
    /// # Arguments
    ///
    /// * `square` - Internal board coordinate
    ///
    /// # Returns
    ///
    /// Standard chess square index (0-63)
    fn map_to_standard_chess_board(&self, square: i16) -> usize {
        // Reverse of map_inner_to_outer_board function
        let board_width = self.board_width;
        let rank = square / board_width;
        let file = square % board_width;

        let chess_rank = rank - 2; // Convert from 2-9 to 0-7
        let chess_file = file - 1; // Convert from 1-8 to 0-7

        (chess_rank * 8 + chess_file) as usize
    }

    /// Updates castling rights based on a move.
    ///
    /// # Arguments
    ///
    /// * `mv` - The move that was made
    fn update_castling_rights(&mut self, mv: &Move) {
        let color = mv.piece.get_color();

        // If king moves, lose both castling rights for that color
        if mv.piece.get_type() == PieceType::King {
            if color == Color::White {
                self.castling_rights.white_kingside = false;
                self.castling_rights.white_queenside = false;
            } else {
                self.castling_rights.black_kingside = false;
                self.castling_rights.black_queenside = false;
            }
        }

        let white_rook_queenside = self.algebraic_to_internal("a1");
        let white_rook_kingside = self.algebraic_to_internal("h1");

        let black_rook_queenside = self.algebraic_to_internal("a8");
        let black_rook_kingside = self.algebraic_to_internal("h8");

        // If rook moves from its starting square, lose corresponding castling right
        match (color, mv.from) {
            (Color::White, square) if square == white_rook_queenside => {
                self.castling_rights.white_queenside = false
            }
            (Color::White, square) if square == white_rook_kingside => {
                self.castling_rights.white_kingside = false
            }
            (Color::Black, square) if square == black_rook_queenside => {
                self.castling_rights.black_queenside = false
            }
            (Color::Black, square) if square == black_rook_kingside => {
                self.castling_rights.black_kingside = false
            }
            _ => {}
        }

        // If a rook is captured, lose corresponding castling right
        if (mv.captured_piece != Piece::EmptySquare)
            && (mv.captured_piece.get_type() == PieceType::Rook)
        {
            match (mv.captured_piece.get_color(), mv.to) {
                (Color::White, square) if square == white_rook_queenside => {
                    self.castling_rights.white_queenside = false
                }
                (Color::White, square) if square == white_rook_kingside => {
                    self.castling_rights.white_kingside = false
                }
                (Color::Black, square) if square == black_rook_queenside => {
                    self.castling_rights.black_queenside = false
                }
                (Color::Black, square) if square == black_rook_kingside => {
                    self.castling_rights.black_kingside = false
                }
                _ => {}
            }
        }

        // If castling move is made, lose both castling rights for that color
        if mv.castling.is_some() {
            if color == Color::White {
                self.castling_rights.white_kingside = false;
                self.castling_rights.white_queenside = false;
            } else {
                self.castling_rights.black_kingside = false;
                self.castling_rights.black_queenside = false;
            }
        }
    }

    /// Checks if kingside castling is legal for the given color.
    ///
    /// Verifies all castling conditions: rights, piece positions, empty squares, and safety.
    ///
    /// # Arguments
    ///
    /// * `chess_board` - Reference to the chess board
    /// * `color` - Color attempting to castle
    /// * `king_square` - Expected king starting square
    /// * `rook_square` - Expected rook starting square
    ///
    /// # Returns
    ///
    /// `true` if kingside castling is legal
    fn can_castle_kingside(&self, color: Color, king_square: i16, rook_square: i16) -> bool {
        // 0. Check if castling privileges are valid
        if (color == Color::White) && (!self.castling_rights.white_kingside) {
            return false;
        }

        if (color == Color::Black) && (!self.castling_rights.black_kingside) {
            return false;
        }

        // 1. Check if king and rook are in starting positions
        if self.get_piece_on_square(king_square)
            != if color == Color::White {
                Piece::WhiteKing
            } else {
                Piece::BlackKing
            }
        {
            return false;
        }

        if self.get_piece_on_square(rook_square)
            != if color == Color::White {
                Piece::WhiteRook
            } else {
                Piece::BlackRook
            }
        {
            return false;
        }

        // 2. Check if squares between king and rook are empty
        let squares_between = match color {
            Color::White => vec![26, 27], // f1, g1
            Color::Black => vec![96, 97], // f8, g8
        };

        for square in squares_between {
            if self.get_piece_on_square(square) != Piece::EmptySquare {
                return false;
            }
        }

        // 3. Check if king is not in check and doesn't move through check
        let check_squares = match color {
            Color::White => vec![25, 26, 27], // e1, f1, g1
            Color::Black => vec![95, 96, 97], // e8, f8, g8
        };

        for square in check_squares {
            let opposite_color = if color == Color::White {
                Color::Black
            } else {
                Color::White
            };
            if self
                .piece_list
                .is_square_attacked(self, square, opposite_color)
            {
                return false;
            }
        }

        true
    }

    /// Checks if queenside castling is legal for the given color.
    ///
    /// Verifies all castling conditions: rights, piece positions, empty squares, and safety.
    ///
    /// # Arguments
    ///
    /// * `chess_board` - Reference to the chess board
    /// * `color` - Color attempting to castle
    /// * `king_square` - Expected king starting square
    /// * `rook_square` - Expected rook starting square
    ///
    /// # Returns
    ///
    /// `true` if queenside castling is legal
    fn can_castle_queenside(&self, color: Color, king_square: i16, rook_square: i16) -> bool {
        // 0. Check if castling privileges are valid
        if (color == Color::White) && (!self.castling_rights.white_queenside) {
            return false;
        }

        if (color == Color::Black) && (!self.castling_rights.black_queenside) {
            return false;
        }

        // 1. Check if king and rook are in starting positions
        if self.get_piece_on_square(king_square)
            != if color == Color::White {
                Piece::WhiteKing
            } else {
                Piece::BlackKing
            }
        {
            return false;
        }

        if self.get_piece_on_square(rook_square)
            != if color == Color::White {
                Piece::WhiteRook
            } else {
                Piece::BlackRook
            }
        {
            return false;
        }

        // 2. Check if squares between king and rook are empty
        let squares_between = match color {
            Color::White => vec![22, 23, 24], // b1, c1, d1
            Color::Black => vec![92, 93, 94], // b8, c8, d8
        };

        for square in squares_between {
            if self.get_piece_on_square(square) != Piece::EmptySquare {
                return false;
            }
        }

        // 3. Check if king is not in check and doesn't move through check
        let check_squares = match color {
            Color::White => vec![25, 24, 23], // e1, d1, c1
            Color::Black => vec![95, 94, 93], // e8, d8, c8
        };

        for square in check_squares {
            let opposite_color = if color == Color::White {
                Color::Black
            } else {
                Color::White
            };
            if self
                .piece_list
                .is_square_attacked(self, square, opposite_color)
            {
                return false;
            }
        }

        true
    }

    fn zobrist_hash(&self, side_to_move: Color) -> u64 {
        let mut hash = 0u64;

        // Hash pieces
        for square_idx in 0..64 {
            let piece = self.get_piece_on_square(self.map_inner_to_outer_board(square_idx));
            if !piece.is_empty() {
                hash ^= self.zobrist.pieces[square_idx as usize][piece as usize];
            }
        }

        // Hash side to move
        if side_to_move == Color::Black {
            hash ^= self.zobrist.side_to_move;
        }

        // Hash castling rights
        if self.castling_rights.white_queenside {
            hash ^= self.zobrist.castling_rights[0];
        }
        if self.castling_rights.white_kingside {
            hash ^= self.zobrist.castling_rights[1];
        }
        if self.castling_rights.black_queenside {
            hash ^= self.zobrist.castling_rights[2];
        }
        if self.castling_rights.black_kingside {
            hash ^= self.zobrist.castling_rights[3];
        }

        // Hash en passant file
        if let Some(square) = self.get_en_passant_target() {
            let file = self.square_file(square) - (self.board_width - 8) / 2;
            hash ^= self.zobrist.en_passant[file as usize];
        }

        hash
    }

    fn update_hash(&mut self, mv: &Move) {
        let from_square = self.map_to_standard_chess_board(mv.from);
        let to_square = self.map_to_standard_chess_board(mv.to);

        // 1. Hash out the piece from its original square
        self.hash ^= self.zobrist.pieces[from_square][mv.piece as usize];

        // 2. Hash out the captured piece from its square (if any)
        if mv.captured_piece.is_valid_piece() {
            self.hash ^= self.zobrist.pieces[to_square][mv.captured_piece as usize];
        }

        // 3. Hash in the moved piece to its new square
        self.hash ^= self.zobrist.pieces[to_square][mv.piece as usize];

        // 4. Hash out the old side to move
        self.hash ^= self.zobrist.side_to_move;

        // 5. Hash out castling move
        if let Some(castling) = &mv.castling {
            let rook_from = self.map_to_standard_chess_board(castling.rook_from);
            let rook_to = self.map_to_standard_chess_board(castling.rook_to);
            self.hash ^= self.zobrist.pieces[rook_from][castling.rook_piece as usize];
            self.hash ^= self.zobrist.pieces[rook_to][castling.rook_piece as usize];
        }

        // 6. Hash out en passant squares
        if let Some(square) = mv.en_passant_square {
            let file = self.square_file(square) - (self.board_width - 8) / 2;
            self.hash ^= self.zobrist.en_passant[file as usize];
        }

        if let Some(square) = mv.previous_en_passant {
            let file = self.square_file(square) - (self.board_width - 8) / 2;
            self.hash ^= self.zobrist.en_passant[file as usize];
        }

        // 7. Hash out en passant moves
        if mv.en_passant {
            let capture_square = if mv.piece.is_white() {
                self.map_to_standard_chess_board(mv.to - self.board_width)
            } else {
                self.map_to_standard_chess_board(mv.to + self.board_width)
            };
            let captured_pawn = if mv.piece.is_white() {
                Piece::BlackPawn
            } else {
                Piece::WhitePawn
            };
            self.hash ^= self.zobrist.pieces[capture_square][captured_pawn as usize];
        }

        // 8. Promotion: Hash out the pawn and hash in the new piece on the same square.
        if let Some(promoted_piece) = mv.promotion {
            // hash out the pawn
            self.hash ^= self.zobrist.pieces[to_square][mv.piece as usize];
            // hash in the promoted piece
            self.hash ^= self.zobrist.pieces[to_square][promoted_piece as usize];
        }

        // 9. Handle castling rights changes
        if let Some(old_rights) = &mv.previous_castling_rights {
            let new_rights = &self.castling_rights;

            // Only update hash for rights that actually changed
            if old_rights.white_queenside != new_rights.white_queenside {
                self.hash ^= self.zobrist.castling_rights[0];
            }
            if old_rights.white_kingside != new_rights.white_kingside {
                self.hash ^= self.zobrist.castling_rights[1];
            }
            if old_rights.black_queenside != new_rights.black_queenside {
                self.hash ^= self.zobrist.castling_rights[2];
            }
            if old_rights.black_kingside != new_rights.black_kingside {
                self.hash ^= self.zobrist.castling_rights[3];
            }
        }
    }

    /// Sets up the board from an 8x8 array of pieces.
    ///
    /// # Arguments
    ///
    /// * `board_position` - Array of 64 pieces representing the chess board
    pub fn set_board(&mut self, board_position: &[Piece; 64], side_to_move: Color) {
        // Set all squares to invalid
        for square in self.board_squares.iter_mut() {
            *square = Piece::SentinelSquare;
        }

        for (square, &piece) in board_position.iter().enumerate() {
            let inner_square = self.map_inner_to_outer_board(square as i16);
            self.set_piece_on_square(piece, inner_square);
        }

        // When the board is set all at once we have to update the piece-lists
        self.piece_list.update_lists(&self.board_squares);

        // Calculate hash for this board position
        self.hash = self.zobrist_hash(side_to_move);
    }

    /// Sets the en passant target square from a standard chess coordinate.
    ///
    /// # Arguments
    ///
    /// * `square` - Standard chess square index (0-63)
    pub fn set_en_passant_square(&mut self, square: i16) {
        self.en_passant_target = Some(self.map_inner_to_outer_board(square));
    }

    /// Sets the castling rights from a CastlingRights struct.
    ///
    /// # Arguments
    ///
    /// * `castling_rights` - New castling rights to set
    pub fn set_castling_rights(&mut self, castling_rights: &CastlingRights) {
        self.castling_rights.white_queenside = castling_rights.white_queenside;
        self.castling_rights.white_kingside = castling_rights.white_kingside;
        self.castling_rights.black_queenside = castling_rights.black_queenside;
        self.castling_rights.black_kingside = castling_rights.black_kingside;
    }

    /// Executes a move on the board.
    ///
    /// Updates the board state, castling rights, and piece lists.
    ///
    /// # Arguments
    ///
    /// * `mv` - The move to execute
    pub fn make_move(&mut self, mv: &Move) {
        self.update_castling_rights(mv);

        let piece = mv.piece;

        // If this was an en passant capture
        if mv.en_passant {
            let capture_square = if mv.piece.is_white() {
                mv.to - self.board_width
            } else {
                mv.to + self.board_width
            };
            self.set_piece_on_square(Piece::EmptySquare, capture_square);
        }

        if let Some(castling) = &mv.castling {
            self.set_piece_on_square(Piece::EmptySquare, castling.rook_from);
            self.set_piece_on_square(castling.rook_piece, castling.rook_to);
        }

        if let Some(piece_promotion) = mv.promotion {
            self.set_piece_on_square(piece_promotion, mv.to);
        } else {
            self.set_piece_on_square(piece, mv.to);
        }

        // When a move is made, the previous square of the piece is cleared
        self.set_piece_on_square(Piece::EmptySquare, mv.from);

        // When pawn moves two squares we update the en passant square
        self.set_en_passant_target(mv.en_passant_square);

        // Update piece list
        self.piece_list.make_move(mv);

        // Update hash AFTER changing board state
        // so we can see what was changed after applying this move
        self.update_hash(mv);
    }

    /// Reverts a move on the board.
    ///
    /// Restores the board state to before the move was made.
    ///
    /// # Arguments
    ///
    /// * `mv` - The move to undo
    pub fn unmake_move(&mut self, mv: &Move) {
        // Update hash BEFORE restoring board state
        // so that we can see what WILL change when this revoked
        self.update_hash(mv);

        // Restaure captured piece
        self.set_piece_on_square(mv.captured_piece, mv.to);

        if mv.en_passant {
            let capture_square = if mv.piece.is_white() {
                mv.to - self.board_width
            } else {
                mv.to + self.board_width
            };
            let captured_pawn = if mv.piece.is_white() {
                Piece::BlackPawn
            } else {
                Piece::WhitePawn
            };
            self.set_piece_on_square(captured_pawn, capture_square);
        }

        if let Some(castling) = &mv.castling {
            self.set_piece_on_square(castling.rook_piece, castling.rook_from);
            self.set_piece_on_square(Piece::EmptySquare, castling.rook_to);
        }

        if let Some(previous_castling_rights) = mv.previous_castling_rights {
            self.castling_rights = previous_castling_rights;
        }

        // Promotion is undone automatically
        self.set_piece_on_square(mv.piece, mv.from);

        // Restore en passant square to previous state
        self.set_en_passant_target(mv.previous_en_passant);

        self.piece_list.unmake_move(mv);
    }

    /// Searches for the best move using minimax with alpha-beta pruning.
    ///
    /// # Arguments
    ///
    /// * `side_to_move` - Color to find the best move for
    ///
    /// # Returns
    ///
    /// `Some(Move)` if a move is found, `None` if no moves available
    pub fn search(&mut self, side_to_move: Color, stop_flag: Arc<AtomicBool>) -> Option<Move> {
        // We clone the board so that the piece-list
        // can do and undo moves to check for legal moves
        let mut board_copy = self.clone();

        let (_, best_move) =
            search::minimax_alpha_beta_search(&mut board_copy, 5, side_to_move, stop_flag);
        best_move
    }

    /// Prints the current board state to stdout.
    ///
    /// Shows the 12x10 internal representation with sentinel squares
    /// and the standard chess board notation.
    pub fn print_board(&self) {
        println!("\n12x10 Chess Board:");
        println!("==============================");
        // Loop over actual board ranks (10 down to 3 in mailbox indexing)
        for rank in (0..12).rev() {
            print!("{:02} │ ", rank - 1);

            for file in 0..10 {
                let idx = (rank * self.board_width + file) as usize;
                let piece = self.board_squares[idx];

                print!("{} ", piece.print_piece());
            }
            println!("│");
        }

        // Print file letters

        println!("   └─────────────────────");
        println!("     z a b c d e f g h i");

        self.piece_list.debug_print();
    }

    /// Debug function to print the raw board array.
    ///
    /// Shows the internal board representation with piece symbols.
    pub fn debug_print(&self) {
        for (square, piece) in self.board_squares.iter().enumerate() {
            print!("{}:{}  ", square, piece.print_piece());
            if square % 10 == 0 {
                println!();
            }
        }
    }

    /// Generates all legal moves for the given color.
    ///
    /// # Arguments
    ///
    /// * `color` - Color to generate moves for
    ///
    /// # Returns
    ///
    /// Vector of legal moves
    pub fn generate_moves(&mut self, color: Color) -> Vec<Move> {
        let mut board_copy = self.clone();
        self.piece_list.generate_legal_moves(&mut board_copy, color)
    }

    /// Create board passing the zobrist keys to be used and the transposition table structure
    pub fn new(zobrist_keys: Arc<Zobrist>, transposition_table: Arc<TranspositionTable>) -> Self {
        ChessBoard {
            board_width: 10,
            board_height: 12,
            board_squares: [Piece::SentinelSquare; 10 * 12],
            en_passant_target: None,

            castling_rights: CastlingRights {
                white_kingside: false,
                white_queenside: false,
                black_kingside: false,
                black_queenside: false,
            },

            piece_list: PieceList::default(),

            zobrist: zobrist_keys,

            hash: 0,

            transposition_table,
        }
    }
}

#[cfg(test)]
mod chess_board_tests {
    use super::*;
    use crate::game_state::GameState;

    fn setup_game_with_fen(fen: &str) -> GameState {
        let zobrist_keys = Arc::new(Zobrist::new());

        let shared_transposition_table = Arc::new(TranspositionTable::new(256));

        let mut game = GameState::new(zobrist_keys, shared_transposition_table);
        game.set_fen_position(fen);
        game
    }

    fn setup_game() -> GameState {
        setup_game_with_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
    }

    #[test]
    fn algebraic_to_internal_convertion() {
        let board = setup_game().board;

        assert_eq!(board.algebraic_to_internal("e4"), 55);
        assert_eq!(board.algebraic_to_internal("a1"), 21);
        assert_eq!(board.algebraic_to_internal("a8"), 91);
        assert_eq!(board.algebraic_to_internal("h1"), 28);
        assert_eq!(board.algebraic_to_internal("h8"), 98);
    }

    fn assert_board_states_equal(b1: &ChessBoard, b2: &ChessBoard, msg: &str) {
        // Compare critical board state components
        assert_eq!(
            b1.castling_rights, b2.castling_rights,
            "{}: Castling rights mismatch",
            msg
        );
        assert_eq!(
            b1.en_passant_target, b2.en_passant_target,
            "{}: En passant target mismatch",
            msg
        );
        assert_eq!(b1.hash, b2.hash, "{}: Hash mismatch", msg);

        // Compare piece positions
        for square in 0..64 {
            let internal_square = b1.map_inner_to_outer_board(square);
            let piece1 = b1.get_piece_on_square(internal_square);
            let piece2 = b2.get_piece_on_square(internal_square);
            assert_eq!(
                piece1, piece2,
                "{}: Piece mismatch at square {}",
                msg, square
            );
        }
    }

    #[test]
    fn test_make_unmake_move() {
        let game =
            setup_game_with_fen("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1");
        let mut board = game.board;
        let original_board = board.clone();

        let mv = board.from_uci("c7c5").unwrap();

        // First make move
        board.make_move(&mv);

        // Undo move
        board.unmake_move(&mv);

        // Board state should be the same
        assert_board_states_equal(&board, &original_board, "test_make_unmake_move");
    }
}

#[cfg(test)]
mod castling_tests {
    use super::*;
    use crate::game_state::GameState;

    fn setup_game_with_fen(fen: &str) -> GameState {
        let zobrist_keys = Arc::new(Zobrist::new());

        let shared_transposition_table = Arc::new(TranspositionTable::new(256));

        let mut game = GameState::new(zobrist_keys, shared_transposition_table);
        game.set_fen_position(fen);
        game
    }

    #[test]
    fn test_castling_move_execution() {
        let mut game = setup_game_with_fen("r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1");

        // Execute kingside castling
        game.make_move("e1g1");
        game.board.print_board();

        // Verify board state after castling
        let king_square = game.board.algebraic_to_internal("g1");
        let rook_square = game.board.algebraic_to_internal("f1");

        assert_eq!(
            game.board.get_piece_on_square(king_square),
            Piece::WhiteKing
        );
        assert_eq!(
            game.board.get_piece_on_square(rook_square),
            Piece::WhiteRook
        );

        // Original squares should be empty
        let original_king = game.board.algebraic_to_internal("e1");
        let original_rook = game.board.algebraic_to_internal("h1");

        assert_eq!(
            game.board.get_piece_on_square(original_king),
            Piece::EmptySquare
        );
        assert_eq!(
            game.board.get_piece_on_square(original_rook),
            Piece::EmptySquare
        );
    }

    #[test]
    fn test_castling_unmake() {
        let mut game = setup_game_with_fen("r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1");

        let initial_board = game.board.board_squares;
        let initial_castling = game.board.castling_rights;

        // Make and unmake castling move
        let mv = game
            .create_move("e1g1")
            .expect("Castling move should be valid");
        game.board.make_move(&mv);
        game.board.unmake_move(&mv);

        // Board should be back to initial state
        assert_eq!(game.board.board_squares, initial_board);
        assert_eq!(game.board.castling_rights, initial_castling);
    }

    #[test]
    fn test_complete_castling_scenario() {
        let mut game = setup_game_with_fen("r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1");

        // White castles kingside
        game.make_move("e1g1");

        // Black castles queenside
        game.make_move("e8c8");

        // Verify final position
        let white_king = game.board.algebraic_to_internal("g1");
        let white_rook = game.board.algebraic_to_internal("f1");
        let black_king = game.board.algebraic_to_internal("c8");
        let black_rook = game.board.algebraic_to_internal("d8");

        assert_eq!(game.board.get_piece_on_square(white_king), Piece::WhiteKing);
        assert_eq!(game.board.get_piece_on_square(white_rook), Piece::WhiteRook);
        assert_eq!(game.board.get_piece_on_square(black_king), Piece::BlackKing);
        assert_eq!(game.board.get_piece_on_square(black_rook), Piece::BlackRook);

        // Castling rights should be lost for both sides
        assert!(!game.board.castling_rights.white_kingside);
        assert!(!game.board.castling_rights.white_queenside);
        assert!(!game.board.castling_rights.black_kingside);
        assert!(!game.board.castling_rights.black_queenside);
    }
}

#[cfg(test)]
mod can_castle_queenside_tests {
    use super::*;
    use crate::game_state::GameState;

    fn setup_game_with_fen(fen: &str) -> GameState {
        let zobrist_keys = Arc::new(Zobrist::new());

        let shared_transposition_table = Arc::new(TranspositionTable::new(256));

        let mut game = GameState::new(zobrist_keys, shared_transposition_table);
        game.set_fen_position(fen);
        game
    }

    #[test]
    fn test_can_castle_queenside_normal() {
        let game = setup_game_with_fen("r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1");

        // White should be able to castle queenside
        assert!(game.board.can_castle_queenside(
            Color::White,
            game.board.algebraic_to_internal("e1"),
            game.board.algebraic_to_internal("a1")
        ));

        // Black should be able to castle queenside
        assert!(game.board.can_castle_queenside(
            Color::Black,
            game.board.algebraic_to_internal("e8"),
            game.board.algebraic_to_internal("a8")
        ));
    }

    #[test]
    fn test_cannot_castle_queenside_if_king_moved() {
        let mut game = setup_game_with_fen("r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1");

        // Simulate king moved by removing castling rights
        game.board.castling_rights.white_queenside = false;

        assert!(!game.board.can_castle_queenside(
            Color::White,
            game.board.algebraic_to_internal("e1"),
            game.board.algebraic_to_internal("a1")
        ));
    }

    #[test]
    fn test_cannot_castle_queenside_if_rook_moved() {
        let mut game = setup_game_with_fen("r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1");

        // Simulate rook moved by removing castling rights
        game.board.castling_rights.white_queenside = false;

        assert!(!game.board.can_castle_queenside(
            Color::White,
            game.board.algebraic_to_internal("e1"),
            game.board.algebraic_to_internal("a1")
        ));
    }

    #[test]
    fn test_cannot_castle_queenside_if_squares_occupied() {
        let game = setup_game_with_fen("r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R1B1K2R w KQkq - 0 1");

        // Bishop on c1 blocks queenside castling
        assert!(!game.board.can_castle_queenside(
            Color::White,
            game.board.algebraic_to_internal("e1"),
            game.board.algebraic_to_internal("a1")
        ));
    }

    #[test]
    fn test_cannot_castle_queenside_through_check() {
        let game = setup_game_with_fen("8/8/8/8/8/2n5/8/R3K3 w - - 0 1");

        // Black knight attacks d1, which king moves through
        assert!(!game.board.can_castle_queenside(
            Color::White,
            game.board.algebraic_to_internal("e1"),
            game.board.algebraic_to_internal("a1")
        ));
    }

    #[test]
    fn test_cannot_castle_queenside_if_in_check() {
        let game = setup_game_with_fen("8/8/8/8/7b/8/8/R3K3 w - - 0 1");

        // Black bishop attacks e1 (king is in check)
        assert!(!game.board.can_castle_queenside(
            Color::White,
            game.board.algebraic_to_internal("e1"),
            game.board.algebraic_to_internal("a1")
        ));
    }

    #[test]
    fn test_cannot_castle_queenside_if_pieces_missing() {
        let game = setup_game_with_fen("4k3/pppppppp/8/8/8/8/PPPPPPPP/4K3 w - - 0 1");

        // No rook on a1
        assert!(!game.board.can_castle_queenside(
            Color::White,
            game.board.algebraic_to_internal("e1"),
            game.board.algebraic_to_internal("a1")
        ));
    }

    #[test]
    fn test_cannot_castle_queenside_wrong_color() {
        let game = setup_game_with_fen("r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1");

        // Black pieces on white squares shouldn't allow white to castle
        assert!(!game.board.can_castle_queenside(
            Color::White,
            game.board.algebraic_to_internal("e1"), // white king
            game.board.algebraic_to_internal("a8")  // black rook - WRONG ROOK!
        ));
    }
}

#[cfg(test)]
mod zobrist_tests {
    use super::*;
    use crate::GameState;

    fn setup_game_with_fen(fen: &str) -> GameState {
        let zobrist_keys = Arc::new(Zobrist::new());

        let shared_transposition_table = Arc::new(TranspositionTable::new(256));

        let mut game = GameState::new(zobrist_keys, shared_transposition_table);
        game.set_fen_position(fen);
        game
    }

    fn create_test_board() -> ChessBoard {
        let zobrist_keys = Arc::new(Zobrist::new());

        let transposition_table = Arc::new(TranspositionTable::new(256));

        let mut game = GameState::new(zobrist_keys, transposition_table);
        game.set_fen_position("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        game.board
    }

    #[test]
    fn test_initial_position_hash_consistency() {
        let board = create_test_board();
        let hash1 = board.zobrist_hash(Color::White);
        let hash2 = board.zobrist_hash(Color::White);

        assert_eq!(hash1, hash2, "Initial position hash should be consistent");
    }

    #[test]
    fn test_hash_changes_with_side_to_move() {
        let board = create_test_board();
        let white_hash = board.zobrist_hash(Color::White);
        let black_hash = board.zobrist_hash(Color::Black);

        assert_ne!(
            white_hash, black_hash,
            "Hash should change with side to move"
        );

        // Test that XORing side_to_move flips correctly
        let side_to_move_key = board.zobrist.side_to_move;
        assert_eq!(
            white_hash ^ side_to_move_key,
            black_hash,
            "XOR side_to_move should flip colors"
        );
    }

    #[test]
    fn test_pawn_move_hash_update() {
        let mut board = create_test_board();
        let initial_hash = board.hash;

        // Create a pawn move (e2 to e4)
        let mv = Move {
            from: board.algebraic_to_internal("e2"),
            to: board.algebraic_to_internal("e4"),
            piece: Piece::WhitePawn,
            captured_piece: Piece::EmptySquare,
            promotion: None,
            castling: None,
            en_passant: false,
            en_passant_square: Some(board.algebraic_to_internal("e3")),
            previous_en_passant: None,
            previous_castling_rights: Some(board.castling_rights.clone()),
        };

        board.update_hash(&mv);
        let after_move_hash = board.hash;

        assert_ne!(
            initial_hash, after_move_hash,
            "Hash should change after pawn move"
        );

        // Test unmake
        board.update_hash(&mv);
        assert_eq!(initial_hash, board.hash, "Hash should restore after unmake");
    }

    #[test]
    fn test_capture_hash_update() {
        // Set up a position where capture is possible
        // For example, after 1.e4 e5 2.Nf3 Nc6 3.Bb5 a6
        // Then capture on c6: Bxc6
        let game = setup_game_with_fen(
            "r1bqkbnr/1ppp1ppp/p1n5/1B2p3/4P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 0 1",
        );

        let mut board = game.board;
        let mv = board.from_uci("b5c6").unwrap();

        let initial_hash = board.hash;

        board.make_move(&mv);

        let after_capture_hash = board.hash;

        assert_ne!(
            initial_hash, after_capture_hash,
            "Hash should change after capture"
        );

        // Test unmake
        board.unmake_move(&mv);
        assert_eq!(
            initial_hash, board.hash,
            "Hash should restore after unmake capture"
        );
    }

    #[test]
    fn test_castling_rights_loss_hash() {
        let game = setup_game_with_fen("r3k2r/pp2pppp/8/8/8/8/PPP2PPP/R3K2R w KQkq - 0 1");

        let mut board = game.board;
        let mv = board.from_uci("e1e2").unwrap();

        let initial_hash = board.hash;

        // King move should remove castling rights
        board.make_move(&mv);

        // Hash should change due to both piece move and castling rights change
        let after_king_move_hash = board.hash;
        assert_ne!(
            initial_hash, after_king_move_hash,
            "Hash should change when castling rights are lost"
        );

        // Test unmake restores original hash
        board.unmake_move(&mv);
        assert_eq!(
            initial_hash, board.hash,
            "Hash should restore after unmaking king move with castling rights change"
        );
    }

    #[test]
    fn test_rook_move_castling_rights_hash() {
        let game = setup_game_with_fen("r3k2r/pp2pppp/8/8/8/8/PPP2PPP/R3K2R w KQkq - 0 1");

        let mut board = game.board;
        let mv = board.from_uci("a1a2").unwrap();

        let initial_hash = board.hash;

        // Moving queenside rook should remove queenside castling right
        board.make_move(&mv);

        let after_rook_move_hash = board.hash;

        assert_ne!(
            initial_hash, after_rook_move_hash,
            "Hash should change when rook move removes castling right"
        );

        board.unmake_move(&mv);
        assert_eq!(
            initial_hash, board.hash,
            "Hash should restore after unmaking rook move"
        );
    }

    #[test]
    fn test_castling_move_hash() {
        let game = setup_game_with_fen("r3k2r/pp2pppp/8/8/8/8/PPP2PPP/R3K2R w KQkq - 0 1");

        let mut board = game.board;
        let mv = board.from_uci("e1g1").unwrap();

        let initial_hash = board.hash;

        board.make_move(&mv);

        let after_castling_hash = board.hash;

        assert_ne!(
            initial_hash, after_castling_hash,
            "Hash should change after castling move"
        );

        board.unmake_move(&mv);
        assert_eq!(
            initial_hash, board.hash,
            "Hash should restore after unmaking castling move"
        );
    }

    #[test]
    fn test_en_passant_hash() {
        let game =
            setup_game_with_fen("rnbqkbnr/pp2pppp/8/2ppP3/8/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 1");

        let mut board = game.board;
        let mv = board.from_uci("e5d6").unwrap(); // Capturing en passant

        let initial_hash = board.hash;

        board.make_move(&mv);

        let after_ep_hash = board.hash;

        assert_ne!(
            initial_hash, after_ep_hash,
            "Hash should change after en passant capture"
        );

        board.unmake_move(&mv);
        assert_eq!(
            initial_hash, board.hash,
            "Hash should restore after unmaking en passant"
        );
    }

    #[test]
    fn test_promotion_hash() {
        // Set up promotion situation - white pawn on 7th rank
        let game = setup_game_with_fen("r4rk1/1p2Pppp/p7/2P1n3/8/B7/P4PPP/R4RK1 b KQkq - 0 1");

        let mut board = game.board;
        let mv = board.from_uci("e7e8q").unwrap();

        let initial_hash = board.hash;

        board.make_move(&mv);

        let after_promotion_hash = board.hash;

        assert_ne!(
            initial_hash, after_promotion_hash,
            "Hash should change after promotion"
        );

        board.unmake_move(&mv);
        assert_eq!(
            initial_hash, board.hash,
            "Hash should restore after unmaking promotion"
        );
    }

    #[test]
    fn test_en_passant_target_file_hash() {
        let mut board = create_test_board();
        let initial_hash = board.hash;

        // Test that setting en passant target file affects hash
        let target_square = board.algebraic_to_internal("e3");
        let file = board.square_file(target_square) - (board.board_width - 8) / 2;

        // XOR in the en passant file
        board.hash ^= board.zobrist.en_passant[file as usize];
        let with_ep_hash = board.hash;

        assert_ne!(
            initial_hash, with_ep_hash,
            "Hash should change when en passant target is set"
        );

        // XOR out to restore
        board.hash ^= board.zobrist.en_passant[file as usize];
        assert_eq!(
            initial_hash, board.hash,
            "Hash should restore when en passant target is cleared"
        );
    }

    #[test]
    fn test_multiple_moves_hash_consistency() {
        let game = setup_game_with_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");

        let mut board = game.board;

        let initial_hash = board.hash;
        // Make a series of moves and unmakes, hash should always restore
        let test_moves = vec!["e2e4", "c7c5", "g1f3", "b8c6"];

        for uci_mv in &test_moves {
            let mv = board.from_uci(uci_mv).unwrap();

            let before_move_hash = board.hash;
            board.make_move(&mv);
            let after_move_hash = board.hash;

            assert_ne!(
                before_move_hash, after_move_hash,
                "Hash should change after each move"
            );

            board.unmake_move(&mv);
            assert_eq!(
                before_move_hash, board.hash,
                "Hash should restore after unmaking each move"
            );
        }

        assert_eq!(
            initial_hash, board.hash,
            "Hash should be back to initial after all moves unmade"
        );
    }

    #[test]
    fn test_zobrist_structure_initialization() {
        let zobrist = Zobrist::new();

        // Verify all arrays are initialized with non-zero values
        assert_ne!(zobrist.side_to_move, 0, "Side to move should be non-zero");

        for i in 0..4 {
            assert_ne!(
                zobrist.castling_rights[i], 0,
                "Castling right {} should be non-zero",
                i
            );
        }

        for i in 0..8 {
            assert_ne!(
                zobrist.en_passant[i], 0,
                "En passant file {} should be non-zero",
                i
            );
        }

        // Check pieces array
        for square in 0..64 {
            for piece in 0..12 {
                assert_ne!(
                    zobrist.pieces[square][piece], 0,
                    "Piece at square {}, type {} should be non-zero",
                    square, piece
                );
            }
        }

        // Verify uniqueness (with high probability)
        let mut values = std::collections::HashSet::new();

        values.insert(zobrist.side_to_move);
        for &val in &zobrist.castling_rights {
            values.insert(val);
        }
        for &val in &zobrist.en_passant {
            values.insert(val);
        }
        for square in 0..64 {
            for piece in 0..12 {
                values.insert(zobrist.pieces[square][piece]);
            }
        }

        // With 64*12 + 4 + 8 + 1 = 781 values, collisions are extremely unlikely
        let expected_unique = 64 * 12 + 4 + 8 + 1;
        assert_eq!(
            values.len(),
            expected_unique,
            "All Zobrist values should be unique"
        );
    }

    #[test]
    fn test_hash_symmetry_operations() {
        let mut board = create_test_board();
        let original_hash = board.hash;

        // Test that XOR operations are symmetric
        let test_value = 0x1234567890ABCDEF;

        board.hash ^= test_value;
        assert_ne!(original_hash, board.hash, "Hash should change after XOR");

        board.hash ^= test_value;
        assert_eq!(
            original_hash, board.hash,
            "Hash should restore after second XOR"
        );
    }
}
