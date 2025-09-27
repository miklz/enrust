//! Chess move representation and conversion utilities.
//!
//! This module provides the Move struct for representing chess moves and
//! conversion functions between different move notations (UCI, algebraic).

use super::piece::{Color, Piece, PieceType};
use crate::game_state::ChessBoard;
use crate::game_state::board::CastlingInfo;
use crate::game_state::board::CastlingRights;

/// Represents a chess move with all associated metadata.
///
/// Stores information about the move itself, captured pieces, special moves
/// (castling, en passant, promotion), and state needed for move unmaking.
#[derive(Clone, Debug, PartialEq)]
pub struct Move {
    /// Starting square of the moving piece
    pub from: i16,
    /// Destination square of the moving piece
    pub to: i16,
    /// The piece being moved
    pub piece: Piece,
    /// Piece captured by this move (EmptySquare if no capture)
    pub captured_piece: Piece,
    /// Promotion piece if this move promotes a pawn
    pub promotion: Option<Piece>,
    /// Castling information if this is a castling move
    pub castling: Option<CastlingInfo>,
    /// Whether this is an en passant capture
    pub en_passant: bool,
    /// En passant target square set by double pawn moves
    pub en_passant_square: Option<i16>,
    /// Previous en passant target for move unmaking
    pub previous_en_passant: Option<i16>,
    /// Previous castling rights for move unmaking
    pub previous_castling_rights: Option<CastlingRights>,
}

impl Move {
    /// Creates a pawn move with pawn-specific metadata.
    ///
    /// Handles pawn moves including promotions, en passant captures, and
    /// double moves that set new en passant targets.
    ///
    /// # Arguments
    ///
    /// * `chess_board` - Reference to the current board state
    /// * `from` - Starting square
    /// * `to` - Destination square
    /// * `piece` - The pawn being moved
    /// * `captured` - Captured piece (if any)
    /// * `promotion` - Promotion piece (for promotions)
    /// * `en_passant` - Whether this is an en passant capture
    /// * `en_passant_square` - New en passant target (for double moves)
    ///
    /// # Returns
    ///
    /// A new Move instance configured for pawn movement
    pub fn create_pawn_move(
        chess_board: &ChessBoard,
        from: i16,
        to: i16,
        piece: Piece,
        captured: Piece,
        promotion: Option<Piece>,
        en_passant: bool,
        en_passant_square: Option<i16>,
    ) -> Self {
        Self {
            from,
            to,
            piece,
            captured_piece: captured,
            promotion,
            castling: None,
            en_passant,
            en_passant_square,
            previous_en_passant: chess_board.get_en_passant_target(),
            previous_castling_rights: Some(chess_board.castling_rights),
        }
    }

    /// Creates a standard move for non-pawn pieces.
    ///
    /// # Arguments
    ///
    /// * `chess_board` - Reference to the current board state
    /// * `from` - Starting square
    /// * `to` - Destination square
    /// * `piece` - The piece being moved
    /// * `captured` - Captured piece (if any)
    ///
    /// # Returns
    ///
    /// A new Move instance for standard piece movement
    pub fn create_move(
        chess_board: &ChessBoard,
        from: i16,
        to: i16,
        piece: Piece,
        captured: Piece,
    ) -> Self {
        Self {
            from,
            to,
            piece,
            captured_piece: captured,
            promotion: None,
            castling: None,
            en_passant: false,
            en_passant_square: None,
            previous_en_passant: chess_board.get_en_passant_target(),
            previous_castling_rights: Some(chess_board.castling_rights),
        }
    }

    /// Creates a castling move with king and rook movement information.
    ///
    /// # Arguments
    ///
    /// * `chess_board` - Reference to the current board state
    /// * `king_from` - King's starting square
    /// * `king_to` - King's destination square
    /// * `king_piece` - The king piece
    /// * `rook_from` - Rook's starting square
    /// * `rook_to` - Rook's destination square
    ///
    /// # Returns
    ///
    /// A new Move instance configured for castling
    pub fn create_castling_move(
        chess_board: &ChessBoard,
        king_from: i16,
        king_to: i16,
        king_piece: Piece,
        rook_from: i16,
        rook_to: i16,
    ) -> Self {
        let color = if king_from == chess_board.algebraic_to_internal("e1") {
            Color::White
        } else {
            Color::Black
        };
        Self {
            from: king_from,
            to: king_to,
            piece: king_piece,
            captured_piece: Piece::EmptySquare,
            promotion: None,
            castling: Some(CastlingInfo {
                rook_from,
                rook_to,
                rook_piece: if color == Color::White {
                    Piece::WhiteRook
                } else {
                    Piece::BlackRook
                },
            }),
            en_passant: false,
            en_passant_square: None,
            previous_en_passant: chess_board.get_en_passant_target(),
            previous_castling_rights: Some(chess_board.castling_rights),
        }
    }

    /// Detects if a move is a castling move based on piece type and squares.
    ///
    /// # Arguments
    ///
    /// * `chess_board` - Reference to the current board state
    /// * `piece` - The piece being moved (must be a king)
    /// * `from` - Starting square
    /// * `to` - Destination square
    ///
    /// # Returns
    ///
    /// `Some(CastlingInfo)` if the move is castling, `None` otherwise
    fn detect_castling(
        chess_board: &ChessBoard,
        piece: Piece,
        from: i16,
        to: i16,
    ) -> Option<CastlingInfo> {
        if piece.get_type() == PieceType::King {
            // Kingside castling: e1-g1 or e8-g8
            let white_king_from = chess_board.algebraic_to_internal("e1");
            let white_king_to = chess_board.algebraic_to_internal("g1");

            let black_king_from = chess_board.algebraic_to_internal("e8");
            let black_king_to = chess_board.algebraic_to_internal("g8");

            if (from == white_king_from && to == white_king_to)
                || (from == black_king_from && to == black_king_to)
            {
                let (rook_from, rook_to) = if from == white_king_from {
                    (
                        chess_board.algebraic_to_internal("h1"),
                        chess_board.algebraic_to_internal("f1"),
                    )
                } else {
                    (
                        chess_board.algebraic_to_internal("h8"),
                        chess_board.algebraic_to_internal("f8"),
                    )
                };
                let rook_piece = if piece.is_white() {
                    Piece::WhiteRook
                } else {
                    Piece::BlackRook
                };
                return Some(CastlingInfo {
                    rook_from,
                    rook_to,
                    rook_piece,
                });
            }

            // Queenside castling: e1-c1 or e8-c8
            let white_king_to = chess_board.algebraic_to_internal("c1");
            let black_king_to = chess_board.algebraic_to_internal("c8");

            if (from == white_king_from && to == white_king_to)
                || (from == black_king_from && to == black_king_to)
            {
                let (rook_from, rook_to) = if from == white_king_from {
                    (
                        chess_board.algebraic_to_internal("a1"),
                        chess_board.algebraic_to_internal("d1"),
                    )
                } else {
                    (
                        chess_board.algebraic_to_internal("a8"),
                        chess_board.algebraic_to_internal("d8"),
                    )
                };
                let rook_piece = if piece.is_white() {
                    Piece::WhiteRook
                } else {
                    Piece::BlackRook
                };
                return Some(CastlingInfo {
                    rook_from,
                    rook_to,
                    rook_piece,
                });
            }
        }
        None
    }

    /// Detects if a move is an en passant capture.
    ///
    /// # Arguments
    ///
    /// * `chess_board` - Reference to the current board state
    /// * `piece` - The piece being moved (must be a pawn)
    /// * `from` - Starting square
    /// * `to` - Destination square
    /// * `captured` - Piece on the destination square
    ///
    /// # Returns
    ///
    /// `true` if the move is an en passant capture, `false` otherwise
    fn detect_en_passant(
        chess_board: &ChessBoard,
        piece: Piece,
        from: i16,
        to: i16,
        captured: Piece,
    ) -> bool {
        // En passant: pawn moving diagonally to empty square when en passant target is set
        if piece.get_type() == PieceType::Pawn && captured == Piece::EmptySquare {
            if let Some(ep_target) = chess_board.get_en_passant_target() {
                // Check if this is an en passant capture
                let expected_from = if piece.is_white() {
                    ep_target - chess_board.board_width // White pawn was one rank below
                } else {
                    ep_target + chess_board.board_width // Black pawn was one rank above
                };

                return from == expected_from && to == ep_target;
            }
        }
        false
    }

    /// Converts an internal board square to algebraic notation.
    ///
    /// # Arguments
    ///
    /// * `chess_board` - Reference to the current board state
    /// * `square` - Internal board coordinate
    ///
    /// # Returns
    ///
    /// Algebraic notation string (e.g., "e4", "a1")
    fn square_to_notation(chess_board: &ChessBoard, square: i16) -> String {
        // Convert from your internal 0-63 representation to algebraic notation
        let chess_square = chess_board.map_to_standard_chess_board(square);
        let file = (chess_square % 8) as u8;
        let rank = (chess_square / 8) as u8;

        let file_char = (b'a' + file) as char;
        let rank_char = (b'1' + rank) as char;

        format!("{}{}", file_char, rank_char)
    }

    /// Converts algebraic notation to a standard chess square index.
    ///
    /// # Arguments
    ///
    /// * `square_notation` - Algebraic notation string (e.g., "e4")
    ///
    /// # Returns
    ///
    /// `Some(i16)` with 0-63 square index if valid, `None` otherwise
    pub fn notation_to_square(square_notation: &str) -> Option<i16> {
        if square_notation.len() != 2 {
            return None;
        }

        let file = square_notation.chars().nth(0).unwrap();
        let rank = square_notation.chars().nth(1).unwrap();

        if !('a'..='h').contains(&file) || !('1'..='8').contains(&rank) {
            return None;
        }

        let file_idx = (file as u8 - b'a') as i16; // a=0, b=1, ...
        let rank_idx = (rank as u8 - b'1') as i16; // 1=0, 2=1, ...

        Some(rank_idx * 8 + file_idx)
    }

    /// Parses a UCI algebraic notation string into a Move struct.
    ///
    /// Supports standard UCI format: `<from><to>[<promotion>]`
    /// Examples: "e2e4", "g1f3", "a7a8q"
    ///
    /// # Arguments
    ///
    /// * `chess_board` - Reference to the current board state
    /// * `uci_notation` - UCI move string
    ///
    /// # Returns
    ///
    /// `Some(Move)` if the notation is valid, `None` otherwise
    pub fn parse_algebraic_move(chess_board: &ChessBoard, uci_notation: &str) -> Option<Self> {
        if uci_notation.len() < 4 {
            return None;
        }

        let from =
            chess_board.map_inner_to_outer_board(Self::notation_to_square(&uci_notation[0..2])?);
        let to =
            chess_board.map_inner_to_outer_board(Self::notation_to_square(&uci_notation[2..4])?);

        // Get the moving piece from the board
        let moving_piece = chess_board.get_piece_on_square(from);
        if moving_piece == Piece::EmptySquare {
            return None;
        }

        // Get captured piece
        let captured_piece = chess_board.get_piece_on_square(to);

        let promotion = if uci_notation.len() == 5 {
            match &uci_notation[4..5] {
                "q" => Some(if moving_piece.is_white() {
                    Piece::WhiteQueen
                } else {
                    Piece::BlackQueen
                }),
                "r" => Some(if moving_piece.is_white() {
                    Piece::WhiteRook
                } else {
                    Piece::BlackRook
                }),
                "n" => Some(if moving_piece.is_white() {
                    Piece::WhiteKnight
                } else {
                    Piece::BlackKnight
                }),
                "b" => Some(if moving_piece.is_white() {
                    Piece::WhiteBishop
                } else {
                    Piece::BlackBishop
                }),
                _ => None,
            }
        } else {
            None
        };

        let castling = Self::detect_castling(chess_board, moving_piece, from, to);

        let en_passant =
            Self::detect_en_passant(chess_board, moving_piece, from, to, captured_piece);

        // Detect en passant target square for double pawn moves
        let en_passant_square = if moving_piece.get_type() == PieceType::Pawn {
            let rank_from = chess_board.square_rank(from);
            let rank_to = chess_board.square_rank(to);
            let rank_diff = rank_to.abs_diff(rank_from);

            // Check if it's a double move forward (2 squares)
            if rank_diff == 2 {
                // Calculate the en passant target square (the square behind the pawn)
                if moving_piece.is_white() {
                    Some(to - chess_board.board_width) // Square behind the pawn (one rank down)
                } else {
                    Some(to + chess_board.board_width) // Square behind the pawn (one rank up)
                }
            } else {
                None
            }
        } else {
            None
        };

        Some(Self {
            from,
            to,
            piece: moving_piece,
            captured_piece,
            promotion,
            castling,
            en_passant,
            en_passant_square,
            previous_en_passant: chess_board.get_en_passant_target(),
            previous_castling_rights: Some(chess_board.castling_rights),
        })
    }

    /// Converts the move to UCI algebraic notation.
    ///
    /// # Arguments
    ///
    /// * `chess_board` - Reference to the current board state
    ///
    /// # Returns
    ///
    /// UCI string representation of the move
    pub fn to_uci(&self, chess_board: &ChessBoard) -> String {
        let from_square = Self::square_to_notation(chess_board, self.from);
        let to_square = Self::square_to_notation(chess_board, self.to);

        let promotion_suffix = if let Some(promo_piece) = self.promotion {
            match promo_piece.get_type() {
                PieceType::Queen => "q",
                PieceType::Rook => "r",
                PieceType::Bishop => "b",
                PieceType::Knight => "n",
                _ => "",
            }
        } else {
            ""
        };

        format!("{}{}{}", from_square, to_square, promotion_suffix)
    }

    /// Checks if this move is a capture.
    ///
    /// # Returns
    ///
    /// `true` if the move captures a piece, `false` otherwise
    pub fn is_capture(&self) -> bool {
        self.captured_piece.is_valid_piece()
    }
}
