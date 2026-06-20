//! Pure minimax search algorithm.
//!
//! Implements the classic minimax algorithm without any optimization
//! techniques. Uses side-relative scoring throughout the tree for
//! compatibility with the default `search()` method.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::game_state::ChessBoard;
use crate::game_state::Color;
use crate::game_state::board::search::SearchAlgorithm;

/// Pure minimax search without any pruning or optimization.
///
/// Recursively evaluates all possible moves to a given depth. Uses side-relative
/// scoring: positive scores favor the side to move, negative favor the opponent.
pub struct PureMinimax;

impl SearchAlgorithm for PureMinimax {
    fn tree_search(
        &self,
        game: &mut ChessBoard,
        depth: u8,
        side_to_move: Color,
        stop_flag: Arc<AtomicBool>,
    ) -> i16 {
        pure_minimax(game, depth, side_to_move, stop_flag)
    }
}

/// Recursive pure minimax evaluation returning a side-relative score.
///
/// White tries to maximize, Black tries to minimize. The score is negated
/// at each recursion level to unify both sides into a single maximization
/// loop — equivalent to the negamax formulation.
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
fn pure_minimax(
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

    match side_to_move {
        Color::White => {
            let mut max_eval = i16::MIN + 1;

            for mv in moves {
                if stop_flag.load(Ordering::Acquire) {
                    return max_eval;
                }

                game.make_move(&mv);
                let eval =
                    -pure_minimax(game, depth - 1, side_to_move.opposite(), stop_flag.clone());
                game.unmake_move(&mv);

                max_eval = max_eval.max(eval);
            }

            max_eval
        }
        Color::Black => {
            let mut max_eval = i16::MIN + 1;

            for mv in moves {
                if stop_flag.load(Ordering::Acquire) {
                    return max_eval;
                }

                game.make_move(&mv);
                let eval =
                    -pure_minimax(game, depth - 1, side_to_move.opposite(), stop_flag.clone());
                game.unmake_move(&mv);

                max_eval = max_eval.max(eval);
            }

            max_eval
        }
    }
}
