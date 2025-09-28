//! Piece list data structure for efficient chess move generation.
//!
//! This module provides the PieceList struct which maintains separate lists
//! for each piece type and color, enabling efficient piece tracking and
//! move generation without scanning the entire board.

use smallvec::{SmallVec, smallvec};
use std::collections::HashMap;

use crate::game_state::board::ChessBoard;
use crate::game_state::board::Color;
use crate::game_state::board::Move;
use crate::game_state::board::Piece;
use crate::game_state::board::PieceType;

/// Maintains separate lists of squares for each piece type and color.
///
/// This data structure provides O(1) access to pieces of a specific type
/// and color, significantly improving move generation performance compared
/// to scanning the entire board.
#[derive(Clone)]
pub struct PieceList {
    /// White king positions (should contain exactly 1 square)
    white_king_list: Vec<i16>,
    /// White queen positions
    white_queen_list: Vec<i16>,
    /// White rook positions
    white_rook_list: Vec<i16>,
    /// White bishop positions
    white_bishop_list: Vec<i16>,
    /// White knight positions
    white_knight_list: Vec<i16>,
    /// White pawn positions
    white_pawn_list: Vec<i16>,

    /// Black king positions (should contain exactly 1 square)
    black_king_list: Vec<i16>,
    /// Black queen positions
    black_queen_list: Vec<i16>,
    /// Black rook positions
    black_rook_list: Vec<i16>,
    /// Black bishop positions
    black_bishop_list: Vec<i16>,
    /// Black knight positions
    black_knight_list: Vec<i16>,
    /// Black pawn positions
    black_pawn_list: Vec<i16>,
}

impl PieceList {
    /// Checks if the king of the given color is in check.
    ///
    /// # Arguments
    ///
    /// * `chess_board` - Reference to the chess board
    /// * `color` - Color to check for check
    ///
    /// # Returns
    ///
    /// Vector of (attacker_piece, attacker_square) tuples if in check, empty otherwise
    pub fn is_king_in_check(&self, chess_board: &ChessBoard, color: Color) -> Vec<(Piece, i16)> {
        let mut attackers = Vec::new();

        if let Some(king) = self.get_king_square(color) {
            attackers.append(&mut self.get_attackers(chess_board, king, color.opposite()));
        }

        attackers
    }

    /// Finds all pieces attacking a given square.
    ///
    /// # Arguments
    ///
    /// * `chess_board` - Reference to the chess board
    /// * `king_square` - Square to check for attacks
    /// * `by_color` - Color of the attacking pieces
    ///
    /// # Returns
    ///
    /// Vector of (attacker_piece, attacker_square) tuples
    fn get_attackers(
        &self,
        chess_board: &ChessBoard,
        king_square: i16,
        by_color: Color,
    ) -> Vec<(Piece, i16)> {
        let mut attackers = Vec::new();

        let attacker_pieces = match by_color {
            Color::White => [
                Piece::WhiteQueen,
                Piece::WhiteRook,
                Piece::WhiteBishop,
                Piece::WhiteKnight,
                Piece::WhitePawn,
                Piece::WhiteKing,
            ],
            Color::Black => [
                Piece::BlackQueen,
                Piece::BlackRook,
                Piece::BlackBishop,
                Piece::BlackKnight,
                Piece::BlackPawn,
                Piece::BlackKing,
            ],
        };

        for attack_piece in attacker_pieces {
            if let Some((attacker_piece, attacker_square)) =
                self.is_attacked_by_piece(chess_board, king_square, attack_piece, by_color)
            {
                attackers.push((attacker_piece, attacker_square));
            }
        }

        attackers
    }

    /// Generates all legal moves for the given color, considering checks and pins.
    ///
    /// This is the main entry point for move generation. It handles:
    /// - Normal moves when not in check
    /// - Check evasion when in single check
    /// - King moves only when in double check
    ///
    /// # Arguments
    ///
    /// * `chess_board` - Mutable reference to the chess board
    /// * `color` - Color to generate moves for
    ///
    /// # Returns
    ///
    /// Vector of legal moves
    pub fn generate_legal_moves(
        &mut self,
        chess_board: &mut ChessBoard,
        color: Color,
    ) -> Vec<Move> {
        let king_attackers = self.is_king_in_check(chess_board, color);

        if king_attackers.is_empty() {
            return self.generate_moves(chess_board, color);
        } else if king_attackers.len() == 1 {
            return self.generate_attacker_captures(chess_board, king_attackers, color);
        } else {
            // If multiple attackers, only king moves are possible
            return self.generate_king_moves(chess_board, color);
        }
    }

    /// Generates moves when the king is in single check.
    ///
    /// Only generates moves that:
    /// - Capture the checking piece
    /// - Block the check (for sliding pieces)
    /// - Move the king out of check
    ///
    /// # Arguments
    ///
    /// * `chess_board` - Mutable reference to the chess board
    /// * `king_attackers` - Information about the checking piece
    /// * `color` - Color to generate moves for
    ///
    /// # Returns
    ///
    /// Vector of legal evasion moves
    fn generate_attacker_captures(
        &mut self,
        chess_board: &mut ChessBoard,
        king_attackers: Vec<(Piece, i16)>,
        color: Color,
    ) -> Vec<Move> {
        let mut valid_moves = Vec::new();

        let Some(king_square) = self.get_king_square(color) else {
            // If there's no king than return empty move list
            return valid_moves;
        };

        let (attacker_piece, attacker_square) = &king_attackers[0];

        // Get squares that block attacker if attacker is a sliding piece
        let mut blocking_squares = vec![*attacker_square];
        match attacker_piece.get_type() {
            PieceType::Queen | PieceType::Rook | PieceType::Bishop => {
                let mut squares = chess_board.get_squares_between(*attacker_square, king_square);

                blocking_squares.append(&mut squares);
            }
            _ => {}
        }

        let pinned_pieces = self.detect_pinned_pieces(chess_board, color);

        // Generate moves for all piece types
        valid_moves = self.generate_queen_moves(chess_board, &pinned_pieces, color);
        valid_moves.append(&mut self.generate_rook_moves(chess_board, &pinned_pieces, color));
        valid_moves.append(&mut self.generate_bishop_moves(chess_board, &pinned_pieces, color));
        valid_moves.append(&mut self.generate_knight_moves(chess_board, &pinned_pieces, color));
        valid_moves.append(&mut self.generate_pawn_moves(chess_board, &pinned_pieces, color));

        // Only consider moves that block the attacker or capture it
        valid_moves.retain(|mv| blocking_squares.contains(&mv.to));
        valid_moves.append(&mut self.generate_king_moves(chess_board, color));

        return valid_moves;
    }

    /// Generates all legal moves for the given color.
    ///
    /// Moves generated here won't let the king in check, but if the king
    /// is already in check it the moves will not account for that.
    ///
    /// # Arguments
    ///
    /// * `chess_board` - Mutable reference to the chess board
    /// * `color` - Color to generate moves for
    ///
    /// # Returns
    ///
    /// Vector of pseudo-legal moves
    fn generate_moves(&mut self, chess_board: &mut ChessBoard, color: Color) -> Vec<Move> {
        let pinned_pieces = self.detect_pinned_pieces(chess_board, color);

        let mut all_moves = self.generate_king_moves(chess_board, color);
        all_moves.append(&mut self.generate_castling_moves(chess_board, color));
        all_moves.append(&mut self.generate_queen_moves(chess_board, &pinned_pieces, color));
        all_moves.append(&mut self.generate_rook_moves(chess_board, &pinned_pieces, color));
        all_moves.append(&mut self.generate_bishop_moves(chess_board, &pinned_pieces, color));
        all_moves.append(&mut self.generate_knight_moves(chess_board, &pinned_pieces, color));
        all_moves.append(&mut self.generate_pawn_moves(chess_board, &pinned_pieces, color));

        all_moves
    }

    /// Generates king moves with safety checks.
    ///
    /// Ensures the king doesn't move into check by temporarily removing
    /// the king and testing if destination squares are attacked.
    ///
    /// # Arguments
    ///
    /// * `chess_board` - Mutable reference to the chess board
    /// * `color` - Color of the king to move
    ///
    /// # Returns
    ///
    /// Vector of legal king moves
    fn generate_king_moves(&mut self, chess_board: &mut ChessBoard, color: Color) -> Vec<Move> {
        let mut moves = Vec::new();
        let (king, king_list) = match color {
            Color::White => (Piece::WhiteKing, &self.white_king_list),
            Color::Black => (Piece::BlackKing, &self.black_king_list),
        };

        let king_rays: [i16; 8] = [
            -1,
            1,
            chess_board.board_width,
            -chess_board.board_width,
            chess_board.board_width + 1,
            -chess_board.board_width + 1,
            chess_board.board_width - 1,
            -chess_board.board_width - 1,
        ];

        for &square in king_list {
            for ray in king_rays {
                let position = square + ray;

                // Remove the king to not have the king blocking a square that would otherwise being attacked
                chess_board.set_piece_on_square(Piece::EmptySquare, square);
                // If king will be in check in this position, don't add to possible moves
                if self.is_square_attacked(chess_board, position, color.opposite()) {
                    // Restore king on the board
                    chess_board.set_piece_on_square(king, square);
                    continue;
                }
                // Restore king on the board
                chess_board.set_piece_on_square(king, square);

                let target = chess_board.get_piece_on_square(position);
                if target.is_empty() || target.is_opponent(color) {
                    moves.push(Move::create_move(
                        chess_board,
                        square,
                        position,
                        king,
                        target,
                    ));
                }
            }
        }

        moves
    }

    /// Generates queen moves considering pin constraints.
    ///
    /// # Arguments
    ///
    /// * `chess_board` - Mutable reference to the chess board
    /// * `pinned_pieces` - Map of pinned pieces and their pin directions
    /// * `color` - Color of the queens to move
    ///
    /// # Returns
    ///
    /// Vector of legal queen moves
    fn generate_queen_moves(
        &mut self,
        chess_board: &mut ChessBoard,
        pinned_pieces: &HashMap<i16, i16>,
        color: Color,
    ) -> Vec<Move> {
        let mut moves = Vec::new();
        let (queen, queen_list) = match color {
            Color::White => (Piece::WhiteQueen, &self.white_queen_list),
            Color::Black => (Piece::BlackQueen, &self.black_queen_list),
        };

        let queen_possible_rays: SmallVec<[i16; 4]> = smallvec![
            -1,
            1,
            chess_board.board_width,
            -chess_board.board_width,
            chess_board.board_width + 1,
            -chess_board.board_width + 1,
            chess_board.board_width - 1,
            -chess_board.board_width - 1,
        ];

        for &square in queen_list {
            let mut queen_rays = queen_possible_rays.clone();

            if let Some(pin_direction) = pinned_pieces.get(&square) {
                queen_rays.retain(|dir| (*dir == *pin_direction) || (*dir == -*pin_direction));
            }

            for ray in &queen_rays {
                let mut position = square + ray;
                loop {
                    let target = chess_board.get_piece_on_square(position);
                    if target.is_empty() {
                        moves.push(Move::create_move(
                            chess_board,
                            square,
                            position,
                            queen,
                            target,
                        ));
                    } else if target.is_opponent(color) {
                        moves.push(Move::create_move(
                            chess_board,
                            square,
                            position,
                            queen,
                            target,
                        ));
                        break;
                    }

                    if target.is_sentinel() || target.is_friend(color) {
                        break;
                    }

                    position += ray;
                }
            }
        }

        moves
    }

    /// Generates rook moves considering pin constraints.
    ///
    /// # Arguments
    ///
    /// * `chess_board` - Mutable reference to the chess board
    /// * `pinned_pieces` - Map of pinned pieces and their pin directions
    /// * `color` - Color of the rooks to move
    ///
    /// # Returns
    ///
    /// Vector of legal rook moves
    fn generate_rook_moves(
        &mut self,
        chess_board: &mut ChessBoard,
        pinned_pieces: &HashMap<i16, i16>,
        color: Color,
    ) -> Vec<Move> {
        let mut moves = Vec::new();

        let (rook, rook_list) = match color {
            Color::White => (Piece::WhiteRook, &self.white_rook_list),
            Color::Black => (Piece::BlackRook, &self.black_rook_list),
        };

        let rook_possible_rays: SmallVec<[i16; 4]> =
            smallvec![1, -1, -chess_board.board_width, chess_board.board_width];

        for &square in rook_list {
            let mut rook_rays = rook_possible_rays.clone();

            if let Some(pin_direction) = pinned_pieces.get(&square) {
                rook_rays.retain(|dir| (*dir == *pin_direction) || (*dir == -*pin_direction));
            }

            for ray in &rook_rays {
                let mut position = square + ray;
                loop {
                    let target = chess_board.get_piece_on_square(position);
                    if target.is_empty() {
                        moves.push(Move::create_move(
                            chess_board,
                            square,
                            position,
                            rook,
                            target,
                        ));
                    } else if target.is_opponent(color) {
                        moves.push(Move::create_move(
                            chess_board,
                            square,
                            position,
                            rook,
                            target,
                        ));
                        // If there is an enemy in this square, the rook can't go further
                        break;
                    }

                    if target.is_sentinel() || target.is_friend(color) {
                        break;
                    }

                    position += ray;
                }
            }
        }

        moves
    }

    /// Generates bishop moves considering pin constraints.
    ///
    /// # Arguments
    ///
    /// * `chess_board` - Mutable reference to the chess board
    /// * `pinned_pieces` - Map of pinned pieces and their pin directions
    /// * `color` - Color of the bishops to move
    ///
    /// # Returns
    ///
    /// Vector of legal bishop moves
    fn generate_bishop_moves(
        &mut self,
        chess_board: &mut ChessBoard,
        pinned_pieces: &HashMap<i16, i16>,
        color: Color,
    ) -> Vec<Move> {
        let mut moves = Vec::new();

        let (bishop, bishop_list) = match color {
            Color::White => (Piece::WhiteBishop, &self.white_bishop_list),
            Color::Black => (Piece::BlackBishop, &self.black_bishop_list),
        };

        let bishop_possible_rays: SmallVec<[i16; 4]> = smallvec![
            chess_board.board_width + 1,
            chess_board.board_width - 1,
            -chess_board.board_width + 1,
            -chess_board.board_width - 1,
        ];

        for &square in bishop_list {
            let mut bishop_rays = bishop_possible_rays.clone();
            // If piece is pinned it can only move along pin direction
            if let Some(pin_direction) = pinned_pieces.get(&square) {
                bishop_rays.retain(|dir| (*dir == *pin_direction) || (*dir == -*pin_direction));
            }

            for ray in &bishop_rays {
                let mut position = square + ray;

                loop {
                    let target = chess_board.get_piece_on_square(position);
                    if target.is_empty() {
                        moves.push(Move::create_move(
                            chess_board,
                            square,
                            position,
                            bishop,
                            target,
                        ));
                    } else if target.is_opponent(color) {
                        moves.push(Move::create_move(
                            chess_board,
                            square,
                            position,
                            bishop,
                            target,
                        ));
                        break;
                    }

                    if target.is_sentinel() || target.is_friend(color) {
                        break;
                    }

                    position += ray;
                }
            }
        }

        moves
    }

    /// Generates knight moves considering pin constraints.
    ///
    /// Knights cannot move if pinned since they jump over other pieces.
    ///
    /// # Arguments
    ///
    /// * `chess_board` - Mutable reference to the chess board
    /// * `pinned_pieces` - Map of pinned pieces and their pin directions
    /// * `color` - Color of the knights to move
    ///
    /// # Returns
    ///
    /// Vector of legal knight moves
    fn generate_knight_moves(
        &mut self,
        chess_board: &mut ChessBoard,
        pinned_pieces: &HashMap<i16, i16>,
        color: Color,
    ) -> Vec<Move> {
        let mut moves = Vec::new();

        let (knight, knight_list) = match color {
            Color::White => (Piece::WhiteKnight, &self.white_knight_list),
            Color::Black => (Piece::BlackKnight, &self.black_knight_list),
        };

        let knight_rays: [i16; 8] = [
            2 * chess_board.board_width + 1,
            2 * chess_board.board_width - 1,
            -2 * chess_board.board_width + 1,
            -2 * chess_board.board_width - 1,
            chess_board.board_width * 1 + 2,
            chess_board.board_width * 1 - 2,
            -chess_board.board_width * 1 + 2,
            -chess_board.board_width * 1 - 2,
        ];

        for &square in knight_list {
            // Knights can't move if pinned (they jump, so any pin makes all moves illegal)
            if pinned_pieces.contains_key(&square) {
                continue;
            }

            for ray in knight_rays {
                let target = chess_board.get_piece_on_square(square + ray);
                if target.is_empty() || target.is_opponent(color) {
                    moves.push(Move::create_move(
                        chess_board,
                        square,
                        square + ray,
                        knight,
                        target,
                    ));
                }
            }
        }

        moves
    }

    /// Generates pawn moves considering pin constraints and special rules.
    ///
    /// Handles pawn moves including:
    /// - Single and double moves
    /// - Captures (including en passant)
    /// - Promotions
    /// - Pin restrictions
    ///
    /// # Arguments
    ///
    /// * `chess_board` - Mutable reference to the chess board
    /// * `pinned_pieces` - Map of pinned pieces and their pin directions
    /// * `color` - Color of the pawns to move
    ///
    /// # Returns
    ///
    /// Vector of legal pawn moves
    fn generate_pawn_moves(
        &mut self,
        chess_board: &mut ChessBoard,
        pinned_pieces: &HashMap<i16, i16>,
        color: Color,
    ) -> Vec<Move> {
        let mut moves = Vec::new();

        let (pawn, pawn_list) = match color {
            Color::White => (Piece::WhitePawn, &self.white_pawn_list),
            Color::Black => (Piece::BlackPawn, &self.black_pawn_list),
        };

        let direction: i16 = match color {
            Color::White => chess_board.board_width,
            Color::Black => -chess_board.board_width,
        };

        let promotion_pieces = match color {
            Color::White => [
                Piece::WhiteQueen,
                Piece::WhiteRook,
                Piece::WhiteBishop,
                Piece::WhiteKnight,
            ],
            Color::Black => [
                Piece::BlackQueen,
                Piece::BlackRook,
                Piece::BlackBishop,
                Piece::BlackKnight,
            ],
        };

        let promotion_rank = match color {
            Color::White => chess_board.square_rank(chess_board.algebraic_to_internal("e8")),
            Color::Black => chess_board.square_rank(chess_board.algebraic_to_internal("e1")),
        };

        let double_push_rank = match color {
            Color::White => chess_board.square_rank(chess_board.algebraic_to_internal("e2")),
            Color::Black => chess_board.square_rank(chess_board.algebraic_to_internal("e7")),
        };

        for &square in pawn_list {
            let mut move_forward = true;
            let mut capture_left = true;
            let mut capture_right = true;

            if let Some(pin_direction) = pinned_pieces.get(&square) {
                // If the pawn is being pinned on horizontal direction
                // there's no possible move for this pawn
                if *pin_direction == 1 || *pin_direction == -1 {
                    continue;
                }

                // If the pawn is being pinned on the file, than it can move in the forward direction
                if (*pin_direction == chess_board.board_width)
                    || (*pin_direction == -chess_board.board_width)
                {
                    capture_left = false;
                    capture_right = false;
                }

                // If the pawn is being pinned on the right diagonal, than there's a capture
                // possibility on the right diagonal
                if (*pin_direction == chess_board.board_width + 1)
                    || (*pin_direction == -chess_board.board_width + 1)
                {
                    move_forward = false;
                    capture_left = false;
                }

                // If the pawn is being pinned on the left diagonal, than there's a capture
                // possibility on the left diagonal
                if (*pin_direction == chess_board.board_width - 1)
                    || (*pin_direction == -chess_board.board_width - 1)
                {
                    move_forward = false;
                    capture_right = false;
                }
            }

            let first_target = chess_board.get_piece_on_square(square + direction);
            if move_forward && first_target.is_empty() {
                if chess_board.square_rank(square + direction) != promotion_rank {
                    moves.push(Move::create_pawn_move(
                        chess_board,
                        square,
                        square + direction,
                        pawn,
                        first_target,
                        None,
                        false,
                        None,
                    ));
                } else {
                    for promotion in promotion_pieces {
                        moves.push(Move::create_pawn_move(
                            chess_board,
                            square,
                            square + direction,
                            pawn,
                            first_target,
                            Some(promotion),
                            false,
                            None,
                        ));
                    }
                }
            }

            let target = chess_board.get_piece_on_square(square + direction + 1);
            if capture_right && target.is_opponent(color) {
                if chess_board.square_rank(square + direction + 1) != promotion_rank {
                    moves.push(Move::create_pawn_move(
                        chess_board,
                        square,
                        square + direction + 1,
                        pawn,
                        target,
                        None,
                        false,
                        None,
                    ));
                } else {
                    for promotion in promotion_pieces {
                        moves.push(Move::create_pawn_move(
                            chess_board,
                            square,
                            square + direction + 1,
                            pawn,
                            target,
                            Some(promotion),
                            false,
                            None,
                        ));
                    }
                }
            } else if capture_right
                && (Some(square + direction + 1) == chess_board.get_en_passant_target())
            {
                // Remove the pawns to not having blocking a piece that would give put the king in check
                chess_board.set_piece_on_square(Piece::EmptySquare, square); // Attacker pawn

                let enemy_pawn = chess_board.get_piece_on_square(square + 1);
                chess_board.set_piece_on_square(Piece::EmptySquare, square + 1); // Captured pawn
                // If king would be in check, don't add to possible moves
                if self.is_king_in_check(chess_board, color).is_empty() {
                    // Move don't let king in check so we add to the possible moves
                    moves.push(Move::create_pawn_move(
                        chess_board,
                        square,
                        square + direction + 1,
                        pawn,
                        target,
                        None,
                        true,
                        None,
                    ));
                }
                // Restore pawns on the board
                chess_board.set_piece_on_square(pawn, square); // Attacker pawn
                chess_board.set_piece_on_square(enemy_pawn, square + 1); // Captured pawn
            }

            let target = chess_board.get_piece_on_square(square + direction - 1);
            if capture_left && target.is_opponent(color) {
                if chess_board.square_rank(square + direction - 1) != promotion_rank {
                    moves.push(Move::create_pawn_move(
                        chess_board,
                        square,
                        square + direction - 1,
                        pawn,
                        target,
                        None,
                        false,
                        None,
                    ));
                } else {
                    for promotion in promotion_pieces {
                        moves.push(Move::create_pawn_move(
                            chess_board,
                            square,
                            square + direction - 1,
                            pawn,
                            target,
                            Some(promotion),
                            false,
                            None,
                        ));
                    }
                }
            } else if capture_left
                && (Some(square + direction - 1) == chess_board.get_en_passant_target())
            {
                // Remove the pawns to not having blocking a piece that would give put the king in check
                chess_board.set_piece_on_square(Piece::EmptySquare, square); // Attacker pawn

                let enemy_pawn = chess_board.get_piece_on_square(square - 1);
                chess_board.set_piece_on_square(Piece::EmptySquare, square - 1); // Captured pawn
                // If king would be in check, don't add to possible moves
                if self.is_king_in_check(chess_board, color).is_empty() {
                    // Move don't let king in check so we add to the possible moves
                    moves.push(Move::create_pawn_move(
                        chess_board,
                        square,
                        square + direction - 1,
                        pawn,
                        target,
                        None,
                        true,
                        None,
                    ));
                }

                // Restore pawns on the board
                chess_board.set_piece_on_square(pawn, square); // Attacker pawn
                chess_board.set_piece_on_square(enemy_pawn, square - 1); // Captured pawn
            }

            let target = chess_board.get_piece_on_square(square + 2 * direction);
            if move_forward
                && (color == Color::White)
                && (chess_board.square_rank(square) == double_push_rank)
            {
                if first_target.is_empty() && target.is_empty() {
                    moves.push(Move::create_pawn_move(
                        chess_board,
                        square,
                        square + 2 * direction,
                        pawn,
                        target,
                        None,
                        false,
                        Some(square + direction),
                    ));
                }
            }

            if move_forward
                && (color == Color::Black)
                && (chess_board.square_rank(square) == double_push_rank)
            {
                if first_target.is_empty() && target.is_empty() {
                    moves.push(Move::create_pawn_move(
                        chess_board,
                        square,
                        square + 2 * direction,
                        pawn,
                        target,
                        None,
                        false,
                        Some(square + direction),
                    ));
                }
            }
        }

        moves
    }

    /// Generates castling moves if legal.
    ///
    /// # Arguments
    ///
    /// * `chess_board` - Reference to the chess board
    /// * `color` - Color to generate castling moves for
    ///
    /// # Returns
    ///
    /// Vector of legal castling moves
    fn generate_castling_moves(&self, chess_board: &ChessBoard, color: Color) -> Vec<Move> {
        let mut moves = Vec::new();

        let (king_square, king_piece, rook_kingside, rook_queenside) = match color {
            Color::White => (
                chess_board.algebraic_to_internal("e1"),
                Piece::WhiteKing,
                chess_board.algebraic_to_internal("h1"),
                chess_board.algebraic_to_internal("a1"),
            ),
            Color::Black => (
                chess_board.algebraic_to_internal("e8"),
                Piece::BlackKing,
                chess_board.algebraic_to_internal("h8"),
                chess_board.algebraic_to_internal("a8"),
            ),
        };

        let castling_rights = &chess_board.castling_rights;

        // Kingside castling
        if (color == Color::White && castling_rights.white_kingside)
            || (color == Color::Black && castling_rights.black_kingside)
        {
            if chess_board.can_castle_kingside(color, king_square, rook_kingside) {
                let king_to = king_square + 2; // g1 or g8
                let rook_to = king_square + 1; // f1 or f8

                moves.push(Move::create_castling_move(
                    chess_board,
                    king_square,
                    king_to,
                    king_piece,
                    rook_kingside,
                    rook_to,
                ));
            }
        }

        // Queenside castling
        if (color == Color::White && castling_rights.white_queenside)
            || (color == Color::Black && castling_rights.black_queenside)
        {
            if chess_board.can_castle_queenside(color, king_square, rook_queenside) {
                let king_to = king_square - 2; // c1 or c8
                let rook_to = king_square - 1; // d1 or d8

                moves.push(Move::create_castling_move(
                    chess_board,
                    king_square,
                    king_to,
                    king_piece,
                    rook_queenside,
                    rook_to,
                ));
            }
        }

        moves
    }

    /// Updates piece lists from the board position.
    ///
    /// Clears all lists and repopulates them by scanning the board.
    /// Used when the board is set up from an external source.
    ///
    /// # Arguments
    ///
    /// * `board_position` - Array of 120 pieces representing the board
    pub fn update_lists(&mut self, board_position: &[Piece; 120]) {
        // The board is our reference, so we can clear all of our lists
        // and set the values from the board to the list
        self.white_pawn_list.clear();
        self.white_rook_list.clear();
        self.white_knight_list.clear();
        self.white_bishop_list.clear();
        self.white_queen_list.clear();
        self.white_king_list.clear();

        self.black_pawn_list.clear();
        self.black_rook_list.clear();
        self.black_knight_list.clear();
        self.black_bishop_list.clear();
        self.black_queen_list.clear();
        self.black_king_list.clear();

        for (square, piece) in board_position.iter().enumerate() {
            // Enumerate returns usize but our squares are i16
            let i16_square = square as i16;
            match piece {
                Piece::WhitePawn => self.white_pawn_list.push(i16_square),
                Piece::WhiteRook => self.white_rook_list.push(i16_square),
                Piece::WhiteKnight => self.white_knight_list.push(i16_square),
                Piece::WhiteBishop => self.white_bishop_list.push(i16_square),
                Piece::WhiteQueen => self.white_queen_list.push(i16_square),
                Piece::WhiteKing => self.white_king_list.push(i16_square),
                Piece::BlackPawn => self.black_pawn_list.push(i16_square),
                Piece::BlackRook => self.black_rook_list.push(i16_square),
                Piece::BlackKnight => self.black_knight_list.push(i16_square),
                Piece::BlackBishop => self.black_bishop_list.push(i16_square),
                Piece::BlackQueen => self.black_queen_list.push(i16_square),
                Piece::BlackKing => self.black_king_list.push(i16_square),
                _ => {}
            }
        }
    }

    /// Applies a move to the piece lists.
    ///
    /// Updates the internal piece lists to reflect the move being made.
    /// Handles captures, promotions, en passant, and castling.
    ///
    /// # Arguments
    ///
    /// * `mv` - The move to apply
    pub fn make_move(&mut self, mv: &Move) {
        // Remove captured piece first (if any)
        if mv.captured_piece != Piece::EmptySquare && mv.captured_piece != Piece::SentinelSquare {
            self.remove_piece(mv.captured_piece, mv.to);
        }

        // Handle en passant separately (captured pawn is on different square)
        if mv.en_passant {
            let capture_square = if mv.piece.is_white() {
                mv.to - 10 // Todo: think of a better way to pass the board width
            } else {
                mv.to + 10
            };
            let captured_pawn = if mv.piece.is_white() {
                Piece::BlackPawn
            } else {
                Piece::WhitePawn
            };
            self.remove_piece(captured_pawn, capture_square);
        }

        // Move the piece
        self.remove_piece(mv.piece, mv.from);

        // Add the piece to its new location (or promoted piece)
        let final_piece = mv.promotion.unwrap_or(mv.piece);
        self.add_piece(final_piece, mv.to);

        // Handle castling
        if let Some(castling) = &mv.castling {
            self.remove_piece(castling.rook_piece, castling.rook_from);
            self.add_piece(castling.rook_piece, castling.rook_to);
        }
    }

    /// Reverts a move in the piece lists.
    ///
    /// Restores the piece lists to their state before the move was made.
    ///
    /// # Arguments
    ///
    /// * `mv` - The move to undo
    pub fn unmake_move(&mut self, mv: &Move) {
        // 1. Handle castling first
        if let Some(castling) = &mv.castling {
            if !self.remove_piece(castling.rook_piece, castling.rook_to) {
                println!("ERROR: Could not remove rook from {}", castling.rook_to);
            }
            self.add_piece(castling.rook_piece, castling.rook_from);
        }

        // 2. Handle en passant
        if mv.en_passant {
            let capture_square = if mv.piece.is_white() {
                mv.to - 10
            } else {
                mv.to + 10
            };
            let captured_pawn = if mv.piece.is_white() {
                Piece::BlackPawn
            } else {
                Piece::WhitePawn
            };

            // Restore en passant capture
            self.add_piece(captured_pawn, capture_square);
        }

        // 3. Remove moved piece (handle promotion)
        let final_piece = mv.promotion.unwrap_or(mv.piece);
        if !self.remove_piece(final_piece, mv.to) {
            println!(
                "ERROR: Could not remove moved piece {} from {}",
                final_piece.print_piece(),
                mv.to
            );
        }

        // 4. Add back the original piece
        self.add_piece(mv.piece, mv.from);

        // 5. Restore captured piece (skip for en passant)
        if !mv.en_passant && mv.captured_piece.is_valid_piece() {
            self.add_piece(mv.captured_piece, mv.to);
        }
    }

    /// Prints the board using piece list information.
    ///
    /// Creates a visual representation of the board based on the piece lists.
    pub fn print_board(&self) {
        // Create an empty 8x8 board
        let mut board = vec!['.'; 64];

        // Helper function to place pieces
        fn place_pieces(board: &mut Vec<char>, pieces: &Vec<i16>, symbol: char) {
            for &square in pieces {
                if square < 64 {
                    board[square as usize] = symbol;
                }
            }
        }

        // Place pieces
        place_pieces(&mut board, &self.white_king_list, 'K');
        place_pieces(&mut board, &self.white_queen_list, 'Q');
        place_pieces(&mut board, &self.white_rook_list, 'R');
        place_pieces(&mut board, &self.white_bishop_list, 'B');
        place_pieces(&mut board, &self.white_knight_list, 'N');
        place_pieces(&mut board, &self.white_pawn_list, 'P');

        place_pieces(&mut board, &self.black_king_list, 'k');
        place_pieces(&mut board, &self.black_queen_list, 'q');
        place_pieces(&mut board, &self.black_rook_list, 'r');
        place_pieces(&mut board, &self.black_bishop_list, 'b');
        place_pieces(&mut board, &self.black_knight_list, 'n');
        place_pieces(&mut board, &self.black_pawn_list, 'p');

        // Print the standard chess board
        println!("\nStandard Chess Board (from Piece Lists):");
        println!("========================================");

        for rank in (0..8).rev() {
            print!("{} │ ", rank + 1);
            for file in 0..8 {
                let index = rank * 8 + file;
                print!("{} ", board[index]);
            }
            println!("│");
        }

        println!("  └─────────────────");
        println!("    a b c d e f g h");
    }

    /// Debug function to show all piece lists.
    ///
    /// Prints the contents of all piece lists for debugging purposes.
    pub fn debug_print(&self) {
        println!("\nPiece List Contents:");
        println!("========================================");

        fn print_list(name: &str, list: &Vec<i16>) {
            let squares: Vec<String> = list.iter().map(|&sq| format!("{}", sq)).collect();
            println!("{:20}: {}", name, squares.join(" "));
        }

        print_list("White Kings", &self.white_king_list);
        print_list("White Queens", &self.white_queen_list);
        print_list("White Rooks", &self.white_rook_list);
        print_list("White Bishops", &self.white_bishop_list);
        print_list("White Knights", &self.white_knight_list);
        print_list("White Pawns", &self.white_pawn_list);

        print_list("Black Kings", &self.black_king_list);
        print_list("Black Queens", &self.black_queen_list);
        print_list("Black Rooks", &self.black_rook_list);
        print_list("Black Bishops", &self.black_bishop_list);
        print_list("Black Knights", &self.black_knight_list);
        print_list("Black Pawns", &self.black_pawn_list);
    }

    /// Adds a piece to the appropriate list in sorted order.
    ///
    /// Uses binary search to maintain sorted order for efficient lookups.
    /// This ensures O(log n) insertion time and maintains list integrity.
    ///
    /// # Arguments
    ///
    /// * `piece` - Piece to add
    /// * `square` - Square where the piece is located
    fn add_piece(&mut self, piece: Piece, square: i16) {
        let list = self.get_list_mut(piece);
        if let Some(list) = list {
            // Insert in sorted order for consistency
            match list.binary_search(&square) {
                Ok(_) => {} // Already exists (shouldn't happen)
                Err(pos) => list.insert(pos, square),
            }
        }
    }
    /// Removes a piece from the appropriate list.
    ///
    /// Uses binary search for efficient O(log n) removal.
    /// Returns whether the piece was successfully found and removed.
    ///
    /// # Arguments
    ///
    /// * `piece` - Piece to remove
    /// * `square` - Square where the piece is located
    ///
    /// # Returns
    ///
    /// `true` if piece was found and removed, `false` otherwise
    fn remove_piece(&mut self, piece: Piece, square: i16) -> bool {
        let list = self.get_list_mut(piece);
        if let Some(list) = list {
            match list.binary_search(&square) {
                Ok(pos) => {
                    list.remove(pos);
                    return true; // Piece found and removed
                }
                Err(_) => {
                    return false; // Doesn't exist (shouldn't happen)
                }
            }
        }
        false // Piece list not found
    }

    /// Gets a mutable reference to the list for a specific piece type.
    ///
    /// # Arguments
    ///
    /// * `piece` - Piece type to get the list for
    ///
    /// # Returns
    ///
    /// Mutable reference to the piece list, or `None` for invalid pieces
    fn get_list_mut(&mut self, piece: Piece) -> Option<&mut Vec<i16>> {
        match piece {
            Piece::WhitePawn => Some(&mut self.white_pawn_list),
            Piece::WhiteRook => Some(&mut self.white_rook_list),
            Piece::WhiteKnight => Some(&mut self.white_knight_list),
            Piece::WhiteBishop => Some(&mut self.white_bishop_list),
            Piece::WhiteQueen => Some(&mut self.white_queen_list),
            Piece::WhiteKing => Some(&mut self.white_king_list),
            Piece::BlackPawn => Some(&mut self.black_pawn_list),
            Piece::BlackRook => Some(&mut self.black_rook_list),
            Piece::BlackKnight => Some(&mut self.black_knight_list),
            Piece::BlackBishop => Some(&mut self.black_bishop_list),
            Piece::BlackQueen => Some(&mut self.black_queen_list),
            Piece::BlackKing => Some(&mut self.black_king_list),
            _ => None,
        }
    }

    /// Gets a reference to the list for a specific piece type.
    ///
    /// # Arguments
    ///
    /// * `piece` - Piece type to get the list for
    ///
    /// # Returns
    ///
    /// Reference to the piece list, or `None` for invalid pieces
    fn get_list(&self, piece: Piece) -> Option<&Vec<i16>> {
        match piece {
            Piece::WhitePawn => Some(&self.white_pawn_list),
            Piece::WhiteRook => Some(&self.white_rook_list),
            Piece::WhiteKnight => Some(&self.white_knight_list),
            Piece::WhiteBishop => Some(&self.white_bishop_list),
            Piece::WhiteQueen => Some(&self.white_queen_list),
            Piece::WhiteKing => Some(&self.white_king_list),
            Piece::BlackPawn => Some(&self.black_pawn_list),
            Piece::BlackRook => Some(&self.black_rook_list),
            Piece::BlackKnight => Some(&self.black_knight_list),
            Piece::BlackBishop => Some(&self.black_bishop_list),
            Piece::BlackQueen => Some(&self.black_queen_list),
            Piece::BlackKing => Some(&self.black_king_list),
            _ => None,
        }
    }

    /// Gets the number of pieces of a specific type on the board.
    ///
    /// # Arguments
    ///
    /// * `piece` - Piece type to count
    ///
    /// # Returns
    ///
    /// Number of pieces, or `None` if the piece type is invalid
    pub fn get_number_of_pieces(&self, piece: Piece) -> Option<i64> {
        if let Some(piece_list) = self.get_list(piece) {
            return Some(piece_list.len() as i64);
        }
        None
    }

    /// Checks if a bishop can attack from one square to another.
    ///
    /// Verifies that the move is diagonal and that no pieces block the path.
    ///
    /// # Arguments
    ///
    /// * `chess_board` - Reference to the chess board
    /// * `from` - Starting square
    /// * `to` - Target square
    ///
    /// # Returns
    ///
    /// `true` if the bishop can legally attack the target square
    fn bishop_attack(chess_board: &ChessBoard, from: i16, to: i16) -> bool {
        let direction = chess_board.get_diagonal_direction(from, to);

        if direction == 0 {
            return false;
        }

        let mut position = from + direction;
        while position != to {
            let piece = chess_board.get_piece_on_square(position);
            if piece.is_empty() {
                position += direction;
                continue;
            } else {
                return false;
            }
        }

        true
    }

    /// Checks if a rook can attack from one square to another.
    ///
    /// Verifies that the move is horizontal/vertical and that no pieces block the path.
    ///
    /// # Arguments
    ///
    /// * `chess_board` - Reference to the chess board
    /// * `from` - Starting square
    /// * `to` - Target square
    ///
    /// # Returns
    ///
    /// `true` if the rook can legally attack the target square
    fn rook_attack(chess_board: &ChessBoard, from: i16, to: i16) -> bool {
        let direction = chess_board.get_rank_or_file_direction(from, to);

        if direction == 0 {
            return false;
        }

        let mut position = from + direction;
        while position != to {
            let piece = chess_board.get_piece_on_square(position);
            if piece.is_empty() {
                position += direction;
                continue;
            } else {
                // Blocked by a piece before reaching destination
                return false;
            }
        }

        true
    }

    /// Checks if a queen can attack from one square to another.
    ///
    /// Combines bishop and rook movement patterns.
    ///
    /// # Arguments
    ///
    /// * `chess_board` - Reference to the chess board
    /// * `from` - Starting square
    /// * `to` - Target square
    ///
    /// # Returns
    ///
    /// `true` if the queen can legally attack the target square
    fn queen_attack(chess_board: &ChessBoard, from: i16, to: i16) -> bool {
        // Check if the queen can move like a bishop to the 'to' square
        let bishop = PieceList::bishop_attack(chess_board, from, to);
        if bishop {
            return true;
        }

        // Check if the queen can move like a rook to the 'to' square
        let rook = PieceList::rook_attack(chess_board, from, to);
        if rook {
            return true;
        }

        // Queen can't move there
        false
    }

    /// Checks if a king can attack from one square to another.
    ///
    /// Kings can only move one square in any direction.
    ///
    /// # Arguments
    ///
    /// * `chess_board` - Reference to the chess board
    /// * `from` - Starting square
    /// * `to` - Target square
    ///
    /// # Returns
    ///
    /// `true` if the king can legally attack the target square
    fn king_attack(chess_board: &ChessBoard, from: i16, to: i16) -> bool {
        if from == to {
            return false;
        }

        let row1 = chess_board.square_rank(from);
        let row2 = chess_board.square_rank(to);

        let col1 = chess_board.square_file(from);
        let col2 = chess_board.square_file(to);

        let row_diff = row1.abs_diff(row2);
        let col_diff = col1.abs_diff(col2);

        if row_diff > 1 || col_diff > 1 {
            return false;
        }

        true
    }

    /// Checks if a knight can attack from one square to another.
    ///
    /// Knights move in an L-shape: 2 squares in one direction and 1 square perpendicular.
    ///
    /// # Arguments
    ///
    /// * `chess_board` - Reference to the chess board
    /// * `from` - Starting square
    /// * `to` - Target square
    ///
    /// # Returns
    ///
    /// `true` if the knight can legally attack the target square
    fn knight_attack(chess_board: &ChessBoard, from: i16, to: i16) -> bool {
        if from == to {
            return false;
        }

        let row1 = chess_board.square_rank(from);
        let row2 = chess_board.square_rank(to);

        let col1 = chess_board.square_file(from);
        let col2 = chess_board.square_file(to);

        let row_diff = row1.abs_diff(row2);
        let col_diff = col1.abs_diff(col2);

        if (row_diff == 2 && col_diff == 1) || (row_diff == 1 && col_diff == 2) {
            return true;
        }

        // Movement doesn't follow an L-shape
        false
    }

    /// Checks if a pawn can attack from one square to another.
    ///
    /// Pawns capture diagonally one square forward.
    ///
    /// # Arguments
    ///
    /// * `chess_board` - Reference to the chess board
    /// * `from` - Starting square
    /// * `to` - Target square
    /// * `color` - Color of the pawn
    ///
    /// # Returns
    ///
    /// `true` if the pawn can legally attack the target square
    fn pawn_attack(chess_board: &ChessBoard, from: i16, to: i16, color: Color) -> bool {
        if from == to {
            return false;
        }

        let row1 = chess_board.square_rank(from);
        let row2 = chess_board.square_rank(to);

        let col1 = chess_board.square_file(from);
        let col2 = chess_board.square_file(to);

        let row_diff = row1.abs_diff(row2);
        let col_diff = col1.abs_diff(col2);

        if col_diff != 1 || row_diff != 1 {
            // Capture can only happen with a square distance,
            return false;
        }

        // Check if pawn direction is right
        if color == Color::White {
            // White pawn always goes up
            if row2 < row1 {
                return false;
            }
        } else {
            // Black pawn always goes down
            if row2 > row1 {
                return false;
            }
        }

        true
    }

    /// Gets the square where the king of the given color is located.
    ///
    /// # Arguments
    ///
    /// * `color` - Color of the king to find
    ///
    /// # Returns
    ///
    /// Square where the king is located, or `None` if not found
    fn get_king_square(&self, color: Color) -> Option<i16> {
        if color == Color::White {
            if let Some(king_list) = self.get_list(Piece::WhiteKing) {
                if let Some(king) = king_list.get(0) {
                    return Some(*king);
                }
            }
            return None;
        } else {
            if let Some(king_list) = self.get_list(Piece::BlackKing) {
                if let Some(king) = king_list.get(0) {
                    return Some(*king);
                }
            }
            return None;
        }
    }

    /// Detects all pieces that are pinned to the king.
    ///
    /// A piece is pinned if moving it would expose the king to check.
    ///
    /// # Arguments
    ///
    /// * `chess_board` - Reference to the chess board
    /// * `color` - Color to check for pinned pieces
    ///
    /// # Returns
    ///
    /// HashMap mapping pinned piece squares to their pin directions
    fn detect_pinned_pieces(&self, chess_board: &ChessBoard, color: Color) -> HashMap<i16, i16> {
        let mut pinned_pieces = HashMap::new();

        let Some(king_square) = self.get_king_square(color) else {
            return pinned_pieces;
        };

        // Check for pins from each direction
        for direction in &[
            -1,
            1, // Horizontal
            -chess_board.board_width,
            chess_board.board_width, // Vertical
            -chess_board.board_width - 1,
            -chess_board.board_width + 1, // Diagonals
            chess_board.board_width - 1,
            chess_board.board_width + 1,
        ] {
            if let Some((pinned_square, pin_direction)) =
                self.find_pinned_piece_in_direction(chess_board, king_square, *direction, color)
            {
                pinned_pieces.insert(pinned_square, pin_direction);
            }
        }

        pinned_pieces
    }

    /// Finds pinned pieces in a specific direction from the king.
    ///
    /// # Arguments
    ///
    /// * `chess_board` - Reference to the chess board
    /// * `king_square` - Square where the king is located
    /// * `direction` - Direction to search for pins
    /// * `color` - Color of the king
    ///
    /// # Returns
    ///
    /// Tuple of (pinned_square, pin_direction) if a pin is found
    fn find_pinned_piece_in_direction(
        &self,
        chess_board: &ChessBoard,
        king_square: i16,
        direction: i16,
        color: Color,
    ) -> Option<(i16, i16)> {
        let mut current = king_square + direction;
        let mut pinned_piece: Option<i16> = None;

        // Move away from king until we hit a piece or board edge
        let mut piece = chess_board.get_piece_on_square(current);
        while !piece.is_sentinel() {
            if !piece.is_empty() {
                if piece.get_color() == color {
                    // First piece we encounter of our color - could be pinned
                    if pinned_piece.is_none() {
                        pinned_piece = Some(current);
                    } else {
                        // Second piece of our color - no pin in this direction
                        return None;
                    }
                } else {
                    // Enemy piece - check if it's a slider that can pin
                    if self.can_piece_pin_in_direction(chess_board, piece, direction) {
                        return pinned_piece.map(|pin_sq| (pin_sq, direction));
                    } else {
                        return None; // Enemy piece can't pin in this direction
                    }
                }
            }

            current += direction;
            piece = chess_board.get_piece_on_square(current);
        }

        None
    }

    /// Checks if a piece can pin in a given direction.
    ///
    /// Only sliding pieces (queen, rook, bishop) can pin.
    ///
    /// # Arguments
    ///
    /// * `chess_board` - Reference to the chess board
    /// * `piece` - Piece to check
    /// * `direction` - Direction of the potential pin
    ///
    /// # Returns
    ///
    /// `true` if the piece can pin in the given direction
    fn can_piece_pin_in_direction(
        &self,
        chess_board: &ChessBoard,
        piece: Piece,
        direction: i16,
    ) -> bool {
        match piece.get_type() {
            PieceType::Queen => true, // Queens can pin in any direction
            PieceType::Rook => {
                // Rooks can pin horizontally or vertically
                direction.abs() == 1 || direction.abs() == chess_board.board_width
            }
            PieceType::Bishop => {
                // Bishops can pin diagonally
                direction.abs() == chess_board.board_width - 1
                    || direction.abs() == chess_board.board_width + 1
            }
            _ => false, // Other pieces can't pin
        }
    }

    /// Checks if a square is attacked by any piece of the given color.
    ///
    /// # Arguments
    ///
    /// * `chess_board` - Reference to the chess board
    /// * `square` - Square to check for attacks
    /// * `by_color` - Color of the attacking pieces
    ///
    /// # Returns
    ///
    /// `true` if the square is attacked by the given color
    pub fn is_square_attacked(
        &self,
        chess_board: &ChessBoard,
        square: i16,
        by_color: Color,
    ) -> bool {
        let attacker_pieces = match by_color {
            Color::White => [
                Piece::WhiteQueen,
                Piece::WhiteRook,
                Piece::WhiteBishop,
                Piece::WhiteKnight,
                Piece::WhitePawn,
                Piece::WhiteKing,
            ],
            Color::Black => [
                Piece::BlackQueen,
                Piece::BlackRook,
                Piece::BlackBishop,
                Piece::BlackKnight,
                Piece::BlackPawn,
                Piece::BlackKing,
            ],
        };

        for attack_piece in attacker_pieces {
            if self
                .is_attacked_by_piece(chess_board, square, attack_piece, by_color)
                .is_some()
            {
                return true;
            }
        }

        false
    }

    /// Checks if a specific piece type attacks a given square.
    ///
    /// # Arguments
    ///
    /// * `chess_board` - Reference to the chess board
    /// * `square` - Square to check for attacks
    /// * `attack_piece` - Type of piece to check
    /// * `by_color` - Color of the attacking pieces
    ///
    /// # Returns
    ///
    /// Some((piece, square)) if attacked, None otherwise
    fn is_attacked_by_piece(
        &self,
        chess_board: &ChessBoard,
        square: i16,
        attack_piece: Piece,
        by_color: Color,
    ) -> Option<(Piece, i16)> {
        if let Some(piece_list) = self.get_list(attack_piece) {
            for &piece_square in piece_list {
                let attacks = match attack_piece.get_type() {
                    PieceType::Queen => Self::queen_attack(chess_board, piece_square, square),
                    PieceType::Rook => Self::rook_attack(chess_board, piece_square, square),
                    PieceType::Bishop => Self::bishop_attack(chess_board, piece_square, square),
                    PieceType::Knight => Self::knight_attack(chess_board, piece_square, square),
                    PieceType::Pawn => {
                        Self::pawn_attack(chess_board, piece_square, square, by_color)
                    }
                    PieceType::King => Self::king_attack(chess_board, piece_square, square),
                };

                if attacks {
                    return Some((attack_piece, piece_square));
                }
            }
        }
        None
    }
}

impl Default for PieceList {
    /// Creates an empty piece list.
    fn default() -> Self {
        PieceList {
            white_king_list: Vec::new(),
            white_queen_list: Vec::new(),
            white_rook_list: Vec::new(),
            white_bishop_list: Vec::new(),
            white_knight_list: Vec::new(),
            white_pawn_list: Vec::new(),

            black_king_list: Vec::new(),
            black_queen_list: Vec::new(),
            black_rook_list: Vec::new(),
            black_bishop_list: Vec::new(),
            black_knight_list: Vec::new(),
            black_pawn_list: Vec::new(),
        }
    }
}

#[cfg(test)]
mod is_square_attacked_tests {
    use super::*;
    use crate::game_state::GameState;

    #[test]
    fn test_pawn_attacks() {
        let mut game = GameState::default();
        game.set_fen_position("8/8/8/8/3p4/4P3/8/8 w - - 0 1");
        /*
            12x10 Chess Board:
            ==============================
            10 │ X X X X X X X X X X │
            09 │ X X X X X X X X X X │
            08 │ X . . . . . . . . X │
            07 │ X . . . . . . . . X │
            06 │ X . . . . . . . . X │
            05 │ X . . . . . . . . X │
            04 │ X . . . p . . . . X │
            03 │ X . . . . P . . . X │
            02 │ X . . . . . . . . X │
            01 │ X . . . . . . . . X │
            00 │ X X X X X X X X X X │
            -1 │ X X X X X X X X X X │
               └─────────────────────
                z a b c d e f g h i
        */

        // Black pawn at d4 should attack e3 and c3
        assert!(game.board.piece_list.is_square_attacked(
            &game.board,
            game.board.algebraic_to_internal("e3"),
            Color::Black
        ));
        assert!(game.board.piece_list.is_square_attacked(
            &game.board,
            game.board.algebraic_to_internal("c3"),
            Color::Black
        ));
        // e4 not attacked
        assert!(!game.board.piece_list.is_square_attacked(
            &game.board,
            game.board.algebraic_to_internal("e4"),
            Color::Black
        ));
        // diagonals behind can't be attacked
        assert!(!game.board.piece_list.is_square_attacked(
            &game.board,
            game.board.algebraic_to_internal("d5"),
            Color::Black
        ));
        assert!(!game.board.piece_list.is_square_attacked(
            &game.board,
            game.board.algebraic_to_internal("c5"),
            Color::Black
        ));
    }

    #[test]
    fn test_knight_attacks() {
        let mut game = GameState::default();
        game.set_fen_position("8/8/8/3N4/8/8/8/8 w - - 0 1");
        /*
            12x10 Chess Board:
            ==============================
            10 │ X X X X X X X X X X │
            09 │ X X X X X X X X X X │
            08 │ X . . . . . . . . X │
            07 │ X . . . . . . . . X │
            06 │ X . . . . . . . . X │
            05 │ X . . . N . . . . X │
            04 │ X . . . . . . . . X │
            03 │ X . . . . . . . . X │
            02 │ X . . . . . . . . X │
            01 │ X . . . . . . . . X │
            00 │ X X X X X X X X X X │
            -1 │ X X X X X X X X X X │
               └─────────────────────
                 z a b c d e f g h i
        */

        // Knight at d5 should attack 8 squares
        let attacked_squares = ["b4", "b6", "c3", "c7", "e3", "e7", "f4", "f6"];

        for &algebraic_square in &attacked_squares {
            let square = game.board.algebraic_to_internal(algebraic_square);
            assert!(
                game.board
                    .piece_list
                    .is_square_attacked(&game.board, square, Color::White),
                "Square {} should be attacked by knight",
                algebraic_square
            );
        }

        let safe_square = game.board.algebraic_to_internal("a1");
        assert!(
            !game
                .board
                .piece_list
                .is_square_attacked(&game.board, safe_square, Color::White),
            "Square {} should not be attacked by knight",
            "a1"
        );
    }

    #[test]
    fn test_bishop_attacks() {
        let mut game = GameState::default();
        game.set_fen_position("8/8/8/3B4/8/8/8/8 w - - 0 1");
        /*
            12x10 Chess Board:
            ==============================
            10 │ X X X X X X X X X X │
            09 │ X X X X X X X X X X │
            08 │ X . . . . . . . . X │
            07 │ X . . . . . . . . X │
            06 │ X . . . . . . . . X │
            05 │ X . . . B . . . . X │
            04 │ X . . . . . . . . X │
            03 │ X . . . . . . . . X │
            02 │ X . . . . . . . . X │
            01 │ X . . . . . . . . X │
            00 │ X X X X X X X X X X │
            -1 │ X X X X X X X X X X │
               └─────────────────────
                 z a b c d e f g h i
        */
        // diagonals
        let attacked_squares = ["a8", "h1", "g8", "a2"];

        // Bishop at d5 should attack diagonals
        for &algebraic_square in &attacked_squares {
            let square = game.board.algebraic_to_internal(algebraic_square);
            assert!(
                game.board
                    .piece_list
                    .is_square_attacked(&game.board, square, Color::White),
                "Square {} should be attacked by bishop",
                algebraic_square
            );
        }

        let safe_square = game.board.algebraic_to_internal("d4");
        assert!(
            !game
                .board
                .piece_list
                .is_square_attacked(&game.board, safe_square, Color::White),
            "Square {} should not be attacked by bishop",
            "d4"
        );
    }

    #[test]
    fn test_rook_attacks() {
        let mut game = GameState::default();
        game.set_fen_position("8/8/8/3R4/8/8/8/8 w - - 0 1");
        /*
            ==============================
            10 │ X X X X X X X X X X │
            09 │ X X X X X X X X X X │
            08 │ X . . . . . . . . X │
            07 │ X . . . . . . . . X │
            06 │ X . . . . . . . . X │
            05 │ X . . . R . . . . X │
            04 │ X . . . . . . . . X │
            03 │ X . . . . . . . . X │
            02 │ X . . . . . . . . X │
            01 │ X . . . . . . . . X │
            00 │ X X X X X X X X X X │
            -1 │ X X X X X X X X X X │
               └─────────────────────
                 z a b c d e f g h i
        */
        let attacked_squares = ["b5", "h5", "d6", "d3"];

        for &algebraic_square in &attacked_squares {
            let square = game.board.algebraic_to_internal(algebraic_square);
            assert!(
                game.board
                    .piece_list
                    .is_square_attacked(&game.board, square, Color::White),
                "Square {} should be attacked by rook",
                algebraic_square
            );
        }
        let safe_square = game.board.algebraic_to_internal("e4");
        assert!(
            !game
                .board
                .piece_list
                .is_square_attacked(&game.board, safe_square, Color::White),
            "Square {} should be attacked by rook",
            "e4"
        );
    }
    #[test]
    fn test_queen_attacks() {
        let mut game = GameState::default();
        game.set_fen_position("8/8/8/3Q4/8/8/8/8 w - - 0 1");
        /*
            12x10 Chess Board:
            ==============================
            10 │ X X X X X X X X X X │
            09 │ X X X X X X X X X X │
            08 │ X . . . . . . . . X │
            07 │ X . . . . . . . . X │
            06 │ X . . . . . . . . X │
            05 │ X . . . Q . . . . X │
            04 │ X . . . . . . . . X │
            03 │ X . . . . . . . . X │
            02 │ X . . . . . . . . X │
            01 │ X . . . . . . . . X │
            00 │ X X X X X X X X X X │
            -1 │ X X X X X X X X X X │
               └─────────────────────
                 z a b c d e f g h i
        */

        let attacked_squares = ["a5", "g5", "d6", "b3", "f7", "b3", "c6", "h1"];

        for &algebraic_square in &attacked_squares {
            let square = game.board.algebraic_to_internal(algebraic_square);
            assert!(
                game.board
                    .piece_list
                    .is_square_attacked(&game.board, square, Color::White),
                "Square {} should be attacked by queen",
                algebraic_square
            );
        }

        let safe_square = game.board.algebraic_to_internal("g4");
        assert!(
            !game
                .board
                .piece_list
                .is_square_attacked(&game.board, safe_square, Color::White)
        );
    }

    #[test]
    fn test_king_attacks() {
        let mut game = GameState::default();
        game.set_fen_position("8/8/8/3K4/8/8/8/8 w - - 0 1");
        /*
            12x10 Chess Board:
            ==============================
            10 │ X X X X X X X X X X │
            09 │ X X X X X X X X X X │
            08 │ X . . . . . . . . X │
            07 │ X . . . . . . . . X │
            06 │ X . . . . . . . . X │
            05 │ X . . . K . . . . X │
            04 │ X . . . . . . . . X │
            03 │ X . . . . . . . . X │
            02 │ X . . . . . . . . X │
            01 │ X . . . . . . . . X │
            00 │ X X X X X X X X X X │
            -1 │ X X X X X X X X X X │
               └─────────────────────
                 z a b c d e f g h i
        */

        // King at d5 should attack surrounding squares
        let attacked_squares = ["c6", "d6", "e6", "c5", "e5", "c4", "d4", "e4"];

        for &algebraic_square in &attacked_squares {
            let square = game.board.algebraic_to_internal(algebraic_square);
            assert!(
                game.board
                    .piece_list
                    .is_square_attacked(&game.board, square, Color::White),
                "Square {} should be attacked by king",
                algebraic_square
            );
        }

        let safe_square = game.board.algebraic_to_internal("e3"); // too far
        assert!(
            !game
                .board
                .piece_list
                .is_square_attacked(&game.board, safe_square, Color::White)
        );
    }

    #[test]
    fn test_blocked_attacks() {
        let mut game = GameState::default();
        game.set_fen_position("8/8/8/3R4/3P4/8/8/8 w - - 0 1");
        /*
            12x10 Chess Board:
            ==============================
            10 │ X X X X X X X X X X │
            09 │ X X X X X X X X X X │
            08 │ X . . . . . . . . X │
            07 │ X . . . . . . . . X │
            06 │ X . . . . . . . . X │
            05 │ X . . . R . . . . X │
            04 │ X . . . P . . . . X │
            03 │ X . . . . . . . . X │
            02 │ X . . . . . . . . X │
            01 │ X . . . . . . . . X │
            00 │ X X X X X X X X X X │
            -1 │ X X X X X X X X X X │
               └─────────────────────
                 z a b c d e f g h i
        */

        // Rook at d5 should not attack squares beyond the pawn at d4
        let attacked_square = game.board.algebraic_to_internal("d4");
        assert!(game.board.piece_list.is_square_attacked(
            &game.board,
            attacked_square,
            Color::White
        ));

        let safe_square = game.board.algebraic_to_internal("d3"); // blocked by pawn
        assert!(
            !game
                .board
                .piece_list
                .is_square_attacked(&game.board, safe_square, Color::White)
        );
    }
}
