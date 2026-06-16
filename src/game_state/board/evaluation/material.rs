//! Material evaluation heuristic.
//!
//! Provides piece value scoring and game phase calculation based on
//! remaining material. Includes bishop pair bonus detection.

use crate::game_state::ChessBoard;
use crate::game_state::Piece;

use super::{GamePhase, HeuristicComponent};

/// Piece values in centipawns for midgame and endgame.
mod values {
    pub const PAWN_MG: i16 = 100;
    pub const PAWN_EG: i16 = 100;
    pub const KNIGHT_MG: i16 = 300;
    pub const KNIGHT_EG: i16 = 300;
    pub const BISHOP_MG: i16 = 300;
    pub const BISHOP_EG: i16 = 300;
    pub const ROOK_MG: i16 = 500;
    pub const ROOK_EG: i16 = 500;
    pub const QUEEN_MG: i16 = 900;
    pub const QUEEN_EG: i16 = 900;
    pub const KING_MG: i16 = 20000;
    pub const KING_EG: i16 = 20000;
    pub const BISHOP_PAIR_MG: i16 = 30;
    pub const BISHOP_PAIR_EG: i16 = 50;
}

/// Heuristic component that evaluates material balance.
///
/// Counts pieces and weights them by standard chess piece values.
/// Applies a tapered bishop pair bonus.
pub struct MaterialHeuristic;

impl HeuristicComponent for MaterialHeuristic {
    fn score(&self, board: &ChessBoard, phase: &GamePhase) -> i16 {
        let piece_list = &board.piece_list;

        let w_pawn = piece_list
            .get_number_of_pieces(Piece::WhitePawn)
            .unwrap_or(0);
        let b_pawn = piece_list
            .get_number_of_pieces(Piece::BlackPawn)
            .unwrap_or(0);
        let w_knight = piece_list
            .get_number_of_pieces(Piece::WhiteKnight)
            .unwrap_or(0);
        let b_knight = piece_list
            .get_number_of_pieces(Piece::BlackKnight)
            .unwrap_or(0);
        let w_bishop = piece_list
            .get_number_of_pieces(Piece::WhiteBishop)
            .unwrap_or(0);
        let b_bishop = piece_list
            .get_number_of_pieces(Piece::BlackBishop)
            .unwrap_or(0);
        let w_rook = piece_list
            .get_number_of_pieces(Piece::WhiteRook)
            .unwrap_or(0);
        let b_rook = piece_list
            .get_number_of_pieces(Piece::BlackRook)
            .unwrap_or(0);
        let w_queen = piece_list
            .get_number_of_pieces(Piece::WhiteQueen)
            .unwrap_or(0);
        let b_queen = piece_list
            .get_number_of_pieces(Piece::BlackQueen)
            .unwrap_or(0);
        let w_king = piece_list
            .get_number_of_pieces(Piece::WhiteKing)
            .unwrap_or(0);
        let b_king = piece_list
            .get_number_of_pieces(Piece::BlackKing)
            .unwrap_or(0);

        let material_mg = values::PAWN_MG * (w_pawn - b_pawn)
            + values::KNIGHT_MG * (w_knight - b_knight)
            + values::BISHOP_MG * (w_bishop - b_bishop)
            + values::ROOK_MG * (w_rook - b_rook)
            + values::QUEEN_MG * (w_queen - b_queen)
            + values::KING_MG * (w_king - b_king);

        let material_eg = values::PAWN_EG * (w_pawn - b_pawn)
            + values::KNIGHT_EG * (w_knight - b_knight)
            + values::BISHOP_EG * (w_bishop - b_bishop)
            + values::ROOK_EG * (w_rook - b_rook)
            + values::QUEEN_EG * (w_queen - b_queen)
            + values::KING_EG * (w_king - b_king);

        let w_bishop_pair = if w_bishop >= 2 {
            values::BISHOP_PAIR_MG
        } else {
            0
        };
        let b_bishop_pair = if b_bishop >= 2 {
            values::BISHOP_PAIR_MG
        } else {
            0
        };
        let pair_mg = w_bishop_pair - b_bishop_pair;

        let w_bishop_pair_eg = if w_bishop >= 2 {
            values::BISHOP_PAIR_EG
        } else {
            0
        };
        let b_bishop_pair_eg = if b_bishop >= 2 {
            values::BISHOP_PAIR_EG
        } else {
            0
        };
        let pair_eg = w_bishop_pair_eg - b_bishop_pair_eg;

        let tapered = super::TaperedScore::new(material_mg + pair_mg, material_eg + pair_eg);

        tapered.interpolate(phase)
    }

    fn delta(&self, _board: &ChessBoard, _mv: &crate::game_state::board::Move) -> Option<i16> {
        None
    }
}
