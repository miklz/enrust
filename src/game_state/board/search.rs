use crate::game_state::ChessBoard;
use crate::game_state::Color;
use crate::game_state::Move;

fn pure_minimax(game: &mut ChessBoard, depth: u64, side_to_move: Color) -> i64 {
    if depth == 0 {
        return game.evaluate();
    }

    let moves = game.generate_moves(side_to_move);

    match side_to_move {
        // Maximizer
        Color::White => {
            let mut max_eval = i64::MIN;

            for mv in moves {
                game.make_move(&mv);
                let eval = pure_minimax(game, depth - 1, side_to_move.opposite());
                game.unmake_move(&mv);

                max_eval = max_eval.max(eval);
            }

            max_eval
        }
        // Minimizer
        Color::Black => {
            let mut min_eval = i64::MAX;

            for mv in moves {
                game.make_move(&mv);
                let eval = pure_minimax(game, depth - 1, side_to_move.opposite());
                game.unmake_move(&mv);

                min_eval = min_eval.min(eval);
            }

            min_eval
        }
    }
}

pub fn pure_minimax_search(game: &mut ChessBoard, depth: u64, side_to_move: Color) -> (i64, Move) {
    let mut best_score = if side_to_move == Color::White {
        i64::MIN
    } else {
        i64::MAX
    };
    let mut best_move = None;

    let moves = game.generate_moves(side_to_move);
    for mv in &moves {
        game.make_move(&mv);
        let score = pure_minimax(game, depth - 1, side_to_move.opposite());
        game.unmake_move(&mv);

        if side_to_move == Color::White {
            if score > best_score {
                best_score = score;
                best_move = Some(mv.clone());
            }
        } else {
            if score < best_score {
                best_score = score;
                best_move = Some(mv.clone());
            }
        }
    }

    // Return best move found, or first move if none
    (best_score, best_move.unwrap_or_else(|| moves[0].clone()))
}

/*
pub fn alpha_beta(
    game: &mut GameBoard,
    depth: u32,
    mut alpha: EvaluationScore,
    beta: EvaluationScore,
    maximizing: bool
) -> EvaluationScore {
    if depth == 0 {
        // Leaf node - return evaluation
        return game.evaluate();
    }

    let moves = game.generate_moves();
    if moves.is_empty() {
        // Checkmate or stalemate
        return if game.is_in_check(game.side_to_move) {
            EvaluationScore::mate_loss(depth) // Negative mate score
        } else {
            EvaluationScore::DRAW // 0
        };
    }

    for mv in moves {
        game.make_move(&mv);
        let score = -alpha_beta(game, depth - 1, -beta, -alpha, !maximizing);
        game.unmake_move(&mv);

        if score >= beta {
            return beta; // Beta cutoff
        }
        if score > alpha {
            alpha = score;
        }
    }

    alpha
}

*/
pub fn quiescence(chess_board: &mut ChessBoard, mut alpha: i64, beta: i64, side_to_move: Color) -> i64 {
    // Evaluate the current (possibly noisy) position
    let stand_pat = chess_board.evaluate();

    // Beta cutoff
    if stand_pat >= beta {
        return beta;
    }

    if stand_pat > alpha {
        alpha = stand_pat;
    }

    // Only consider capture moves
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