//! Chess search algorithms and evaluation functions.
//!
//! This module implements various chess search algorithms including minimax,
//! negamax, alpha-beta pruning, and quiescence search for stable positions.
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::game_state::ChessBoard;
use crate::game_state::Color;
use crate::game_state::Move;

/// Pure minimax algorithm without optimization techniques.
///
/// Recursively evaluates all possible moves to a given depth using the
/// minimax algorithm. White tries to maximize the score while Black tries
/// to minimize it.
///
/// # Arguments
///
/// * `game` - Mutable reference to the chess board
/// * `depth` - Search depth (number of plies to look ahead)
/// * `side_to_move` - Color of the player to move
///
/// # Returns
///
/// Evaluation score from the perspective of the side to move
fn pure_minimax(
    game: &mut ChessBoard,
    depth: u64,
    side_to_move: Color,
    stop_flag: Arc<AtomicBool>,
) -> i64 {
    if depth == 0 {
        return game.evaluate();
    }

    let moves = game.generate_moves(side_to_move);

    match side_to_move {
        // Maximizer (White)
        Color::White => {
            let mut max_eval = i64::MIN;

            for mv in moves {
                // Abruptly end the search if required
                if stop_flag.load(Ordering::Acquire) {
                    return max_eval;
                }

                game.make_move(&mv);
                let eval =
                    pure_minimax(game, depth - 1, side_to_move.opposite(), stop_flag.clone());
                game.unmake_move(&mv);

                max_eval = max_eval.max(eval);
            }

            max_eval
        }
        // Minimizer (Black)
        Color::Black => {
            let mut min_eval = i64::MAX;

            for mv in moves {
                // Abruptly end the search if required
                if stop_flag.load(Ordering::Acquire) {
                    return min_eval;
                }

                game.make_move(&mv);
                let eval =
                    pure_minimax(game, depth - 1, side_to_move.opposite(), stop_flag.clone());
                game.unmake_move(&mv);

                min_eval = min_eval.min(eval);
            }

            min_eval
        }
    }
}

/// Performs a complete minimax search and returns the best move.
///
/// # Arguments
///
/// * `game` - Mutable reference to the chess board
/// * `depth` - Search depth (number of plies to look ahead)
/// * `side_to_move` - Color of the player to move
///
/// # Returns
///
/// Tuple containing the best evaluation score and the best move found
pub fn pure_minimax_search(
    game: &mut ChessBoard,
    depth: u64,
    side_to_move: Color,
    stop_flag: Arc<AtomicBool>,
) -> (i64, Option<Move>) {
    let mut best_score = if side_to_move == Color::White {
        i64::MIN
    } else {
        i64::MAX
    };
    let mut best_move = None;

    let moves = game.generate_moves(side_to_move);
    for mv in &moves {
        // Abruptly end the search if required
        if stop_flag.load(Ordering::Acquire) {
            return (best_score, best_move);
        }

        game.make_move(mv);
        let score = pure_minimax(game, depth - 1, side_to_move.opposite(), stop_flag.clone());
        game.unmake_move(mv);

        if side_to_move == Color::White {
            if score >= best_score {
                best_score = score;
                best_move = Some(mv.clone());
            }
        } else if score <= best_score {
            best_score = score;
            best_move = Some(mv.clone());
        }
    }

    // Return best move found, or none if no moves available
    (best_score, best_move)
}

/// Negamax implementation of the minimax algorithm.
///
/// Uses a single recursive function for both players by negating scores
/// at each recursion level. More elegant than separate min/max functions.
///
/// # Arguments
///
/// * `game` - Mutable reference to the chess board
/// * `depth` - Search depth (negative for negamax convention)
/// * `side_to_move` - Color of the player to move
///
/// # Returns
///
/// Evaluation score from the perspective of the side to move
fn pure_negamax(
    game: &mut ChessBoard,
    depth: i64,
    side_to_move: Color,
    stop_flag: Arc<AtomicBool>,
) -> i64 {
    if depth == 0 {
        let perspective = if side_to_move == Color::White { 1 } else { -1 };
        return game.evaluate() * perspective;
    }

    let moves = game.generate_moves(side_to_move);
    let mut score = i64::MIN + 1; // +1 to avoid overflow when negated

    for mv in &moves {
        // Abruptly end the search if required
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

/// Performs a complete negamax search and returns the best move.
///
/// # Arguments
///
/// * `game` - Mutable reference to the chess board
/// * `depth` - Search depth (number of plies to look ahead)
/// * `side_to_move` - Color of the player to move
///
/// # Returns
///
/// Tuple containing the best evaluation score and the best move found
pub fn pure_negamax_search(
    game: &mut ChessBoard,
    depth: i64,
    side_to_move: Color,
    stop_flag: Arc<AtomicBool>,
) -> (i64, Option<Move>) {
    let mut best_move = None;
    let mut best_score = i64::MIN;

    let moves = game.generate_moves(side_to_move);

    for mv in &moves {
        // Abruptly end the search if required
        if stop_flag.load(Ordering::Acquire) {
            return (best_score, best_move);
        }

        game.make_move(mv);
        let score = -pure_negamax(game, depth - 1, side_to_move.opposite(), stop_flag.clone());
        game.unmake_move(mv);
        if score >= best_score {
            best_move = Some(mv.clone());
            best_score = score;
        }
    }

    let perspective = if side_to_move == Color::White { 1 } else { -1 };
    best_score *= perspective;

    // Return best move found, or none if no moves available
    (best_score, best_move)
}

/// Minimax algorithm with alpha-beta pruning for improved performance.
///
/// Alpha-beta pruning eliminates branches that cannot influence the final
/// decision, significantly reducing the number of positions evaluated.
///
/// # Arguments
///
/// * `game` - Mutable reference to the chess board
/// * `depth` - Search depth (number of plies to look ahead)
/// * `alpha` - Alpha value for pruning (best value maximizer can guarantee)
/// * `beta` - Beta value for pruning (best value minimizer can guarantee)
/// * `side_to_move` - Color of the player to move
/// * `stop_flag` - Search control to force the end of search at any time
///
/// # Returns
///
/// Evaluation score from the perspective of the side to move
fn minimax_alpha_beta(
    game: &mut ChessBoard,
    depth: u64,
    mut alpha: i64,
    mut beta: i64,
    side_to_move: Color,
    stop_flag: Arc<AtomicBool>,
) -> i64 {
    // Terminal node check - reached maximum depth
    if depth == 0 {
        return game.evaluate();
    }

    let moves = game.generate_moves(side_to_move);

    if side_to_move == Color::White {
        let mut max_eval = i64::MIN;

        for mv in moves {
            // Abruptly end the search if required
            if stop_flag.load(Ordering::Acquire) {
                return max_eval;
            }

            game.make_move(&mv);
            let eval = minimax_alpha_beta(
                game,
                depth - 1,
                alpha,
                beta,
                side_to_move.opposite(),
                stop_flag.clone(),
            );
            game.unmake_move(&mv);

            max_eval = max_eval.max(eval);
            alpha = alpha.max(eval);

            // Beta cutoff - Black won't allow this line
            if beta <= alpha {
                break;
            }
        }

        max_eval
    } else {
        let mut min_eval = i64::MAX;

        for mv in moves {
            // Abruptly end the search if required
            if stop_flag.load(Ordering::Acquire) {
                return min_eval;
            }

            game.make_move(&mv);
            let eval = minimax_alpha_beta(
                game,
                depth - 1,
                alpha,
                beta,
                side_to_move.opposite(),
                stop_flag.clone(),
            );
            game.unmake_move(&mv);

            min_eval = min_eval.min(eval);
            beta = beta.min(eval);

            // Alpha cutoff - White won't allow this line
            if beta <= alpha {
                break;
            }
        }

        min_eval
    }
}

/// Performs a complete alpha-beta search and returns the best move.
///
/// This is the primary search function used by the chess engine.
///
/// # Arguments
///
/// * `game` - Mutable reference to the chess board
/// * `depth` - Search depth (number of plies to look ahead)
/// * `side_to_move` - Color of the player to move
///
/// # Returns
///
/// Tuple containing the best evaluation score and the best move found
pub fn minimax_alpha_beta_search(
    game: &mut ChessBoard,
    depth: u64,
    side_to_move: Color,
    stop_flag: Arc<AtomicBool>,
) -> (i64, Option<Move>) {
    let mut best_score = if side_to_move == Color::White {
        i64::MIN
    } else {
        i64::MAX
    };
    let mut best_move = None;

    let mut alpha = i64::MIN;
    let mut beta = i64::MAX;

    let moves = game.generate_moves(side_to_move);

    for mv in moves {
        // Abruptly end the search if required
        if stop_flag.load(Ordering::Acquire) {
            return (best_score, best_move);
        }

        game.make_move(&mv);
        let score = minimax_alpha_beta(
            game,
            depth - 1,
            i64::MIN,
            i64::MAX,
            side_to_move.opposite(),
            stop_flag.clone(),
        );
        game.unmake_move(&mv);

        if side_to_move == Color::White {
            if score >= best_score {
                best_score = score;
                best_move = Some(mv.clone());
                alpha = alpha.max(score)
            }
        } else if score <= best_score {
            best_score = score;
            best_move = Some(mv.clone());
            beta = beta.min(score);
        }
    }

    (best_score, best_move)
}

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
    mut alpha: i64,
    beta: i64,
    side_to_move: Color,
) -> i64 {
    // Evaluate the current (possibly noisy) position
    let stand_pat = chess_board.evaluate();

    // Beta cutoff - position is already good enough for the opponent
    if stand_pat >= beta {
        return beta;
    }

    // Update alpha if current position is better than known alpha
    if stand_pat > alpha {
        alpha = stand_pat;
    }

    // Only consider capture moves (quiets are skipped in quiescence search)
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
