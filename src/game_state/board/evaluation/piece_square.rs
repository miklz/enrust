//! Piece-square table heuristic based on PesTO's values.
//!
//! Uses tapered evaluation: midgame and endgame PSTs are stored separately
//! and interpolated based on the current game phase.
//!
//! Tables are laid out from black's perspective: index 0 = a8, index 63 = h1.
//! Lookup uses the formula `(7 - rank) * 8 + file` for both colors.

use crate::game_state::ChessBoard;
use crate::game_state::Piece;

use super::{GamePhase, HeuristicComponent, TaperedScore};

type Pst = [i16; 64];

const PAWN_MG: Pst = build_pawn_mg();
const PAWN_EG: Pst = build_pawn_eg();
const KNIGHT_MG: Pst = build_knight_mg();
const KNIGHT_EG: Pst = build_knight_eg();
const BISHOP_MG: Pst = build_bishop_mg();
const BISHOP_EG: Pst = build_bishop_eg();
const ROOK_MG: Pst = build_rook_mg();
const ROOK_EG: Pst = build_rook_eg();
const QUEEN_MG: Pst = build_queen_mg();
const QUEEN_EG: Pst = build_queen_eg();
const KING_MG: Pst = build_king_mg();
const KING_EG: Pst = build_king_eg();

/// Looks up a PST value for a piece at the given standard chess square.
///
/// The PST tables are stored in PesTO published format (index 0 = a8).
/// White pieces use the XOR 56 flip (mirroring the rank), black pieces
/// use the table directly.
///
/// # Arguments
///
/// * `pst` - The piece-square table (PesTO format: index 0 = a8)
/// * `sq` - Standard chess square (0 = a1, 63 = h8)
/// * `is_white` - Whether the piece is white (affects mirroring)
///
/// # Returns
///
/// The PST value for that square from the piece's perspective.
fn pst_lookup(pst: &Pst, sq: i16, is_white: bool) -> i16 {
    let idx = if is_white { sq ^ 56 } else { sq };
    pst[idx as usize]
}

/// Maps an internal 12x10 mailbox coordinate to a standard 0-63 square.
fn to_standard(board: &ChessBoard, internal_sq: i16) -> i16 {
    board.map_to_standard_chess_board(internal_sq) as i16
}

/// Heuristic component that evaluates piece placement using PesTO PSTs.
///
/// For each piece on the board, looks up its midgame and endgame PST
/// values and interpolates them based on the current game phase.
pub struct PieceSquareHeuristic;

impl HeuristicComponent for PieceSquareHeuristic {
    fn score(&self, board: &ChessBoard, phase: &GamePhase) -> i16 {
        let mut total = 0i16;

        board.piece_list.for_each_piece(|piece, sq| {
            let std_sq = to_standard(board, sq);
            let (mg, eg) = pst_value(piece, std_sq);

            total += if piece.is_white() { 1 } else { -1 }
                * TaperedScore::new(mg, eg).interpolate(phase);
        });

        total
    }

    fn delta(&self, _board: &ChessBoard, _mv: &crate::game_state::board::Move) -> Option<i16> {
        None
    }
}

fn pst_value(piece: Piece, sq: i16) -> (i16, i16) {
    let is_white = piece.is_white();
    match piece {
        Piece::WhitePawn | Piece::BlackPawn => (
            pst_lookup(&PAWN_MG, sq, is_white),
            pst_lookup(&PAWN_EG, sq, is_white),
        ),
        Piece::WhiteKnight | Piece::BlackKnight => (
            pst_lookup(&KNIGHT_MG, sq, is_white),
            pst_lookup(&KNIGHT_EG, sq, is_white),
        ),
        Piece::WhiteBishop | Piece::BlackBishop => (
            pst_lookup(&BISHOP_MG, sq, is_white),
            pst_lookup(&BISHOP_EG, sq, is_white),
        ),
        Piece::WhiteRook | Piece::BlackRook => (
            pst_lookup(&ROOK_MG, sq, is_white),
            pst_lookup(&ROOK_EG, sq, is_white),
        ),
        Piece::WhiteQueen | Piece::BlackQueen => (
            pst_lookup(&QUEEN_MG, sq, is_white),
            pst_lookup(&QUEEN_EG, sq, is_white),
        ),
        Piece::WhiteKing | Piece::BlackKing => (
            pst_lookup(&KING_MG, sq, is_white),
            pst_lookup(&KING_EG, sq, is_white),
        ),
        _ => (0, 0),
    }
}

const fn build_pawn_mg() -> Pst {
    [
        0, 0, 0, 0, 0, 0, 0, 0, 98, 134, 61, 95, 68, 126, 34, -11, -6, 7, 26, 31, 65, 56, 25, -20,
        -14, 13, 6, 21, 23, 12, 17, -23, -27, -2, -5, 12, 17, 6, 10, -25, -26, -4, -4, -10, 3, 3,
        33, -12, -35, -1, -20, -23, -15, 24, 38, -22, 0, 0, 0, 0, 0, 0, 0, 0,
    ]
}

const fn build_pawn_eg() -> Pst {
    [
        0, 0, 0, 0, 0, 0, 0, 0, 178, 173, 158, 134, 147, 132, 165, 187, 94, 100, 85, 67, 56, 53,
        82, 84, 32, 24, 13, 5, -2, 4, 17, 17, 13, 9, -3, -7, -7, -8, 3, -1, 4, 7, -6, 1, 0, -5, -1,
        -8, 13, 8, 8, 10, 13, 0, 2, -7, 0, 0, 0, 0, 0, 0, 0, 0,
    ]
}

const fn build_knight_mg() -> Pst {
    [
        -167, -89, -34, -49, 61, -97, -15, -107, -73, -41, 72, 36, 23, 62, 7, -17, -47, 60, 37, 65,
        84, 129, 73, 44, -9, 17, 19, 53, 37, 69, 18, 22, -13, 4, 16, 13, 28, 19, 21, -8, -23, -9,
        12, 10, 19, 17, 25, -16, -29, -53, -12, -3, -1, 18, -14, -19, -105, -21, -58, -33, -17,
        -28, -19, -23,
    ]
}

const fn build_knight_eg() -> Pst {
    [
        -58, -38, -13, -28, -31, -27, -63, -99, -25, -8, -25, -2, -9, -25, -24, -52, -24, -20, 10,
        9, -1, -9, -19, -41, -17, 3, 22, 22, 22, 11, 8, -18, -18, -6, 16, 25, 16, 17, 4, -18, -23,
        -3, -1, 15, 10, -3, -20, -22, -42, -20, -10, -5, -2, -20, -23, -44, -29, -51, -23, -15,
        -22, -18, -50, -64,
    ]
}

const fn build_bishop_mg() -> Pst {
    [
        -29, 4, -82, -37, -25, -42, 7, -8, -26, 16, -18, -13, 30, 59, 18, -47, -16, 37, 43, 40, 35,
        50, 37, -2, -4, 5, 19, 50, 37, 37, 7, -2, -6, 13, 13, 26, 34, 12, 10, 4, 0, 15, 15, 15, 14,
        27, 18, 10, 4, 15, 16, 0, 7, 21, 33, 1, -33, -3, -14, -21, -13, -12, -39, -21,
    ]
}

const fn build_bishop_eg() -> Pst {
    [
        -14, -21, -11, -8, -7, -9, -17, -24, -8, -4, 7, -12, -3, -13, -4, -14, 2, -8, 0, -1, -2, 6,
        0, 4, -3, 9, 12, 9, 14, 10, 3, 2, -6, 3, 13, 19, 7, 10, -3, -9, -12, -3, 8, 10, 13, 3, -7,
        -15, -14, -18, -7, -1, 4, -9, -15, -27, -23, -9, -23, -5, -9, -16, -5, -17,
    ]
}

const fn build_rook_mg() -> Pst {
    [
        32, 42, 32, 51, 63, 9, 31, 43, 27, 32, 58, 62, 80, 67, 26, 44, -5, 19, 26, 36, 17, 45, 61,
        16, -24, -11, 7, 26, 24, 35, -8, -20, -36, -26, -12, -1, 9, -7, 6, -23, -45, -25, -16, -17,
        3, 0, -5, -33, -44, -16, -20, -9, -1, 11, -6, -71, -19, -13, 1, 17, 16, 7, -37, -26,
    ]
}

const fn build_rook_eg() -> Pst {
    [
        13, 10, 18, 15, 12, 12, 8, 5, 11, 13, 13, 11, -3, 3, 8, 3, 7, 7, 7, 5, 4, -3, -5, -3, 4, 3,
        13, 1, 2, 1, -1, 2, 3, 5, 8, 4, -5, -6, -8, -11, -4, 0, -5, -1, -7, -12, -8, -16, -6, -6,
        0, 2, -9, -9, -11, -3, -9, 2, 3, -1, -5, -13, 4, -20,
    ]
}

const fn build_queen_mg() -> Pst {
    [
        -28, 0, 29, 12, 59, 44, 43, 45, -24, -39, -5, 1, -16, 57, 28, 54, -13, -17, 7, 8, 29, 56,
        47, 57, -27, -27, -16, -16, -1, 17, -2, 1, -9, -26, -9, -10, -2, -4, 3, -3, -14, 2, -11,
        -2, -5, 2, 14, 5, -35, -8, 11, 2, 8, 15, -3, 1, -1, -18, -9, 10, -15, -25, -31, -50,
    ]
}

const fn build_queen_eg() -> Pst {
    [
        -9, 22, 22, 27, 27, 19, 10, 20, -17, 20, 32, 41, 58, 25, 30, 0, -20, 6, 9, 49, 47, 35, 19,
        9, 3, 22, 24, 45, 57, 40, 57, 36, -18, 28, 19, 47, 31, 34, 39, 23, -16, -27, 15, 6, 9, 17,
        10, 5, -22, -23, -30, -16, -16, -23, -36, -32, -33, -28, -22, -43, -5, -32, -20, -41,
    ]
}

const fn build_king_mg() -> Pst {
    [
        -65, 23, 16, -15, -56, -34, 2, 13, 29, -1, -20, -7, -8, -4, -38, -29, -9, 24, 2, -16, -20,
        6, 22, -22, -17, -20, -12, -27, -30, -25, -14, -36, -49, -1, -27, -39, -46, -44, -33, -51,
        -14, -14, -22, -46, -44, -30, -15, -27, 1, 7, -8, -64, -43, -16, 9, 8, -15, 36, 12, -54, 8,
        -28, 24, 14,
    ]
}

const fn build_king_eg() -> Pst {
    [
        -74, -35, -18, -18, -11, 15, 4, -17, -12, 17, 14, 17, 17, 38, 23, 11, 10, 17, 23, 15, 20,
        45, 44, 13, -8, 22, 24, 27, 26, 33, 26, 3, -18, -4, 21, 24, 27, 23, 9, -11, -19, -3, 11,
        21, 23, 16, 7, -9, -27, -11, 4, 13, 14, 4, -5, -17, -53, -34, -21, -11, -28, -14, -24, -43,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pst_lookup_a1() {
        let sq = 0i16;
        assert_eq!(pst_lookup(&KING_MG, sq, true), -15);
        assert_eq!(pst_lookup(&KING_EG, sq, true), -53);
    }

    #[test]
    fn test_pst_lookup_a8() {
        let sq = 56i16;
        assert_eq!(pst_lookup(&KING_MG, sq, true), -65);
        assert_eq!(pst_lookup(&KING_EG, sq, true), -74);
    }

    #[test]
    fn test_pst_lookup_e4() {
        let sq = 28i16;
        assert_eq!(pst_lookup(&KNIGHT_MG, sq, true), 28);
        assert_eq!(pst_lookup(&KNIGHT_EG, sq, true), 16);
    }
}
