//! Minimax with alpha-beta pruning search algorithm.
//!
//! Implements alpha-beta pruning on top of the minimax algorithm using the
//! negamax formulation. Uses side-relative scoring throughout for compatibility
//! with the default `search()` implementation.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::game_state::ChessBoard;
use crate::game_state::Color;
use crate::game_state::Move;
use crate::game_state::board::search::SearchAlgorithm;
use crate::game_state::board::transposition_table::{NodeType, TranspositionTableData};

/// Minimax search with alpha-beta pruning and transposition table support.
///
/// Uses the negamax formulation: a single recursive function for both players
/// with side-relative scoring. Alpha/beta bounds are negated at each recursion
/// level. Provides transposition table probing and capture-based move ordering.
pub struct MinimaxAlphaBeta;

impl SearchAlgorithm for MinimaxAlphaBeta {
    fn tree_search(
        &self,
        board: &mut ChessBoard,
        depth: u8,
        side_to_move: Color,
        stop_flag: Arc<AtomicBool>,
    ) -> i16 {
        minimax_alpha_beta(
            board,
            depth,
            i16::MIN + 1,
            i16::MAX,
            side_to_move,
            stop_flag,
        )
    }
}

/// Recursive negamax search with alpha-beta pruning and transposition table.
///
/// Returns a side-relative score (positive = good for `side_to_move`).
/// Alpha and beta are side-relative bounds from the current player's perspective.
///
/// # Arguments
///
/// * `board` - Mutable reference to the chess board
/// * `depth` - Remaining search depth in plies
/// * `alpha` - Lower bound (best score current side can guarantee)
/// * `beta` - Upper bound (best score opponent can force)
/// * `side_to_move` - Color of the player to move
/// * `stop_flag` - Atomic flag to abort the search early
///
/// # Returns
///
/// Side-relative evaluation score
fn minimax_alpha_beta(
    board: &mut ChessBoard,
    depth: u8,
    mut alpha: i16,
    beta: i16,
    side_to_move: Color,
    stop_flag: Arc<AtomicBool>,
) -> i16 {
    let original_alpha = alpha;
    let mut tt_move = None;

    {
        let tt = &board.transposition_table;
        if let Some(position) = tt.retrieve_position(board.hash)
            && position.depth >= depth
        {
            match position.node_type {
                NodeType::Exact => return position.score,
                NodeType::UpperBound => {
                    if position.score <= alpha {
                        return position.score;
                    }
                }
                NodeType::LowerBound => {
                    if position.score >= beta {
                        return position.score;
                    }
                }
            }
            tt_move = Move::decode(position.best_move, board);
        }
    }

    if depth == 0 {
        let perspective = if side_to_move == Color::White { 1 } else { -1 };
        return board.evaluate() * perspective;
    }

    let mut best_move = None;
    let mut moves = board.generate_moves(side_to_move);

    moves.sort_by(|mv_a, mv_b| {
        let mv_a_is_capture = mv_a.is_capture();
        let mv_b_is_capture = mv_b.is_capture();

        match (mv_a_is_capture, mv_b_is_capture) {
            (true, false) => std::cmp::Ordering::Greater,
            (false, true) => std::cmp::Ordering::Less,
            _ => std::cmp::Ordering::Equal,
        }
    });

    if let Some(mv) = tt_move.clone() {
        moves.push(mv);
    }

    for mv in moves.into_iter().rev() {
        if stop_flag.load(Ordering::Acquire) {
            return alpha;
        }

        board.make_move(&mv);
        let score = -minimax_alpha_beta(
            board,
            depth - 1,
            -beta,
            -alpha,
            side_to_move.opposite(),
            stop_flag.clone(),
        );
        board.unmake_move(&mv);

        if score > alpha {
            alpha = score;
            best_move = Some(mv);
        }

        if alpha >= beta {
            break;
        }
    }

    let node_type = if alpha <= original_alpha {
        NodeType::UpperBound
    } else if alpha >= beta {
        NodeType::LowerBound
    } else {
        NodeType::Exact
    };

    let encoded_move = if let Some(mv) = best_move {
        mv.encode(board)
    } else {
        0
    };

    let tt = &board.transposition_table;
    tt.save_position(
        board.hash,
        &TranspositionTableData {
            depth,
            score: alpha,
            node_type,
            best_move: encoded_move,
            age: 0,
        },
    );

    alpha
}
