//! Chess search trait, orchestration strategies, and algorithm modules.
//!
//! This module defines a two-layer search architecture:
//!
//! 1. **`SearchAlgorithm`** — low-level recursive tree search. Each algorithm
//!    implements [`SearchAlgorithm::tree_search`] returning a side-relative score.
//!    The root-level move iteration is provided by the default
//!    [`SearchAlgorithm::search`] method.
//! 2. **`Search`** — high-level orchestration (depth-first, iterative deepening).
//!
//! These layers are independent: any `SearchAlgorithm` can be plugged into any
//! `Search` orchestrator without modification.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::game_state::ChessBoard;
use crate::game_state::Color;
use crate::game_state::Move;

pub mod minimax_alpha_beta;
pub mod pure_minimax;
pub mod pure_negamax;
pub mod quiescence;

pub use minimax_alpha_beta::MinimaxAlphaBeta;
pub use pure_minimax::PureMinimax;
pub use pure_negamax::PureNegamax;

/// Low-level recursive tree search algorithm.
///
/// Implementations provide [`tree_search`](Self::tree_search) to recursively
/// evaluate the game tree at a given depth. The [`search`](Self::search) method
/// has a default implementation that iterates over root moves and calls
/// `tree_search` on each child position.
pub trait SearchAlgorithm {
    /// Recursively traverse the game tree to the given depth, returning a
    /// side-relative score (positive = good for `side_to_move`).
    ///
    /// # Arguments
    ///
    /// * `board` - Mutable reference to the chess board
    /// * `depth` - Search depth in plies
    /// * `side_to_move` - Color of the player to move
    /// * `stop_flag` - Atomic flag to abort the search early
    ///
    /// # Returns
    ///
    /// Side-relative evaluation score
    fn tree_search(
        &self,
        board: &mut ChessBoard,
        depth: u8,
        side_to_move: Color,
        stop_flag: Arc<AtomicBool>,
    ) -> i16;

    /// Search for the best move at the root level.
    ///
    /// The default implementation iterates over root moves, makes each one,
    /// calls [`tree_search`](Self::tree_search) on the resulting position,
    /// and tracks the best move found.
    ///
    /// # Arguments
    ///
    /// * `board` - Mutable reference to the chess board
    /// * `depth` - Search depth in plies
    /// * `side_to_move` - Color of the player to move
    /// * `stop_flag` - Atomic flag to abort the search early
    ///
    /// # Returns
    ///
    /// Tuple containing the best evaluation score (white-centric) and the
    /// best move found
    fn search(
        &self,
        board: &mut ChessBoard,
        depth: u8,
        side_to_move: Color,
        stop_flag: Arc<AtomicBool>,
    ) -> (i16, Option<Move>) {
        let moves = board.generate_moves(side_to_move);
        let mut best_move: Option<Move> = None;
        let mut best_score: Option<i16> = None;

        for mv in moves {
            if stop_flag.load(Ordering::Acquire) {
                if let Some(score) = best_score {
                    let white_score = if side_to_move == Color::White {
                        score
                    } else {
                        -score
                    };
                    return (white_score, best_move);
                }
                return (0, None);
            }

            board.make_move(&mv);
            let score =
                -self.tree_search(board, depth - 1, side_to_move.opposite(), stop_flag.clone());
            board.unmake_move(&mv);

            if best_score.is_none() || score > best_score.unwrap() {
                best_score = Some(score);
                best_move = Some(mv.clone());
            }
        }

        let best_score = best_score.unwrap_or(0);
        let white_score = if side_to_move == Color::White {
            best_score
        } else {
            -best_score
        };

        (white_score, best_move)
    }
}

/// High-level search strategy that orchestrates a [`SearchAlgorithm`].
///
/// Strategies control how the algorithm is invoked — single-shot depth-first
/// or across multiple iterations for iterative deepening.
pub trait Search {
    /// Perform the search and return the best move.
    ///
    /// # Arguments
    ///
    /// * `board` - Mutable reference to the chess board
    /// * `side_to_move` - Color of the player to move
    /// * `stop_flag` - Atomic flag to abort the search early
    ///
    /// # Returns
    ///
    /// Tuple containing the best evaluation score and the best move found
    fn search(
        &self,
        board: &mut ChessBoard,
        side_to_move: Color,
        stop_flag: Arc<AtomicBool>,
    ) -> (i16, Option<Move>);
}

/// Single-shot search at a fixed depth.
///
/// Delegates directly to the wrapped algorithm at `max_depth` with no
/// iterative deepening.
pub struct DepthFirst<A: SearchAlgorithm> {
    max_depth: u8,
    algorithm: A,
}

impl<A: SearchAlgorithm> DepthFirst<A> {
    /// Creates a new depth-first search strategy.
    ///
    /// # Arguments
    ///
    /// * `algorithm` - The tree search algorithm to use
    /// * `max_depth` - Fixed search depth in plies
    pub fn new(algorithm: A, max_depth: u8) -> Self {
        DepthFirst {
            max_depth,
            algorithm,
        }
    }
}

impl<A: SearchAlgorithm> Search for DepthFirst<A> {
    fn search(
        &self,
        board: &mut ChessBoard,
        side_to_move: Color,
        stop_flag: Arc<AtomicBool>,
    ) -> (i16, Option<Move>) {
        self.algorithm
            .search(board, self.max_depth, side_to_move, stop_flag)
    }
}

/// Iterative deepening search strategy.
///
/// Searches from depth 1 up to `max_depth`, reusing the best move from
/// the previous iteration as a starting point. Each iteration restarts
/// the underlying algorithm at the progressively deeper depth.
pub struct IterativeDeepening<A: SearchAlgorithm> {
    max_depth: u8,
    algorithm: A,
}

impl<A: SearchAlgorithm> IterativeDeepening<A> {
    /// Creates a new iterative deepening search strategy.
    ///
    /// # Arguments
    ///
    /// * `algorithm` - The tree search algorithm to use
    /// * `max_depth` - Maximum search depth in plies
    pub fn new(algorithm: A, max_depth: u8) -> Self {
        IterativeDeepening {
            max_depth,
            algorithm,
        }
    }
}

impl<A: SearchAlgorithm> Search for IterativeDeepening<A> {
    fn search(
        &self,
        board: &mut ChessBoard,
        side_to_move: Color,
        stop_flag: Arc<AtomicBool>,
    ) -> (i16, Option<Move>) {
        let mut best_move = None;
        let mut best_score = if side_to_move == Color::White {
            i16::MIN
        } else {
            i16::MAX
        };

        for depth in 1..=self.max_depth {
            if stop_flag.load(Ordering::Acquire) {
                break;
            }
            let (score, mv) = self
                .algorithm
                .search(board, depth, side_to_move, stop_flag.clone());
            best_score = score;
            best_move = mv.or(best_move);
        }

        (best_score, best_move)
    }
}
