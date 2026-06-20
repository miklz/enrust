//! Quiescence search to stabilize evaluations in tactical positions.
//!
//! Extends search beyond the normal depth limit to only consider captures
//! and other forcing moves, preventing horizon effect problems where
//! tactical sequences extend beyond the search depth.

use crate::game_state::ChessBoard;
use crate::game_state::Color;

/// Quiescence search to stabilize evaluations in tactical positions.
///
/// Extends search beyond the normal depth limit to only consider captures
/// and other forcing moves, preventing horizon effect problems where
/// tactical sequences extend beyond the search depth.
///
/// # Arguments
///
/// * `chess_board` - Mutable reference to the chess board
/// * `alpha` - Alpha value for pruning
/// * `beta` - Beta value for pruning
/// * `side_to_move` - Color of the player to move
///
/// # Returns
///
/// Stabilized evaluation score after considering captures
pub fn quiescence(
    chess_board: &mut ChessBoard,
    mut alpha: i16,
    beta: i16,
    side_to_move: Color,
) -> i16 {
    let stand_pat = chess_board.evaluate();

    if stand_pat >= beta {
        return beta;
    }

    if stand_pat > alpha {
        alpha = stand_pat;
    }

    let captures = chess_board
        .generate_moves(side_to_move)
        .into_iter()
        .filter(|mv| mv.is_capture())
        .collect::<Vec<_>>();

    for mv in captures {
        chess_board.make_move(&mv);
        let score = -quiescence(chess_board, -beta, -alpha, side_to_move.opposite());
        chess_board.unmake_move(&mv);

        if score >= beta {
            return beta;
        }
        if score > alpha {
            alpha = score;
        }
    }

    alpha
}
