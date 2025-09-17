use super::piece::{Color, Piece, PieceType};
use crate::game_state::ChessBoard;
use crate::game_state::board::CastlingInfo;
use crate::game_state::board::CastlingRights;

#[derive(Clone)]
pub struct Move {
    pub from: i16,
    pub to: i16,
    pub piece: Piece,                                     // The moving piece
    pub captured_piece: Piece, // Piece captured (Empty squares count as pieces)
    pub promotion: Option<Piece>, // Promotion piece (if any)
    pub castling: Option<CastlingInfo>, // Castling information
    pub en_passant: bool,      // Whether this is an en passant capture
    pub en_passant_square: Option<i16>, // Set when pawn moves two squares
    pub previous_en_passant: Option<i16>, // Previous en passant target
    pub previous_castling_rights: Option<CastlingRights>, // Previous castling rights
}

impl Move {
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

    fn square_to_notation(chess_board: &ChessBoard, square: i16) -> String {
        // Convert from your internal 0-63 representation to algebraic notation
        let chess_square = chess_board.map_to_standard_chess_board(square);
        let file = (chess_square % 8) as u8;
        let rank = (chess_square / 8) as u8;

        let file_char = (b'a' + file) as char;
        let rank_char = (b'1' + rank) as char;

        format!("{}{}", file_char, rank_char)
    }

    /* Convert <rank><file> to 8x8 square */
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

    /* Convert uci algebraic notation format:
     * <from square><to square>[<promoted to>]
     * to Move struct
     */
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

        Some(Self {
            from,
            to,
            piece: moving_piece,
            captured_piece,
            promotion,
            castling,
            en_passant,
            en_passant_square: None,
            previous_en_passant: chess_board.get_en_passant_target(),
            previous_castling_rights: Some(chess_board.castling_rights),
        })
    }

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
}
