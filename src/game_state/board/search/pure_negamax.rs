//! Pure negamax search algorithm.
//!
//! Implements the negamax formulation of the minimax algorithm, which uses
//! a single recursive function for both players by negating scores at each
//! recursion level. Side-relative scoring throughout.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::game_state::ChessBoard;
use crate::game_state::Color;
use crate::game_state::board::search::SearchAlgorithm;

/// Pure negamax search without any pruning or optimization.
///
/// Uses a single recursive function for both players by negating scores
/// at each recursion level. Positive scores favor the side to move.
pub struct PureNegamax;

impl SearchAlgorithm for PureNegamax {
    fn tree_search(
        &self,
        game: &mut ChessBoard,
        depth: u8,
        side_to_move: Color,
        stop_flag: Arc<AtomicBool>,
    ) -> i16 {
        pure_negamax(game, depth, side_to_move, stop_flag)
    }
}

/// Recursive pure negamax evaluation returning a side-relative score.
///
/// # Arguments
///
/// * `game` - Mutable reference to the chess board
/// * `depth` - Search depth (number of plies to look ahead)
/// * `side_to_move` - Color of the player to move
/// * `stop_flag` - Flag to abort search early
///
/// # Returns
///
/// Side-relative evaluation score (positive = good for the side to move)
fn pure_negamax(
    game: &mut ChessBoard,
    depth: u8,
    side_to_move: Color,
    stop_flag: Arc<AtomicBool>,
) -> i16 {
    if depth == 0 {
        let perspective = if side_to_move == Color::White { 1 } else { -1 };
        return game.evaluate() * perspective;
    }

    let moves = game.generate_moves(side_to_move);
    let mut score = i16::MIN + 1;

    for mv in &moves {
        if stop_flag.load(Ordering::Acquire) {
            return score;
        }

        game.make_move(mv);
        score = score.max(-pure_negamax(
            game,
            depth - 1,
            side_to_move.opposite(),
            stop_flag.clone(),
        ));
        game.unmake_move(mv);
    }

    score
}
