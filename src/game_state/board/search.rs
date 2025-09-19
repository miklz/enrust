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

pub fn pure_minimax_search(game: &mut ChessBoard, depth: u64, side_to_move: Color) -> (i64, Option<Move>) {
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

    // Return best move found, or none
    (best_score, best_move)
}

fn pure_negamax(game: &mut ChessBoard, depth: i64, side_to_move: Color) -> i64 {
    if depth == 0 {
        let perspective = if side_to_move == Color::White {1} else {-1};
        return game.evaluate() * perspective;
    }

    let moves = game.generate_moves(side_to_move);
    let mut score = i64::MIN + 1; // +1 to not overflow when negated

    for mv in &moves {
        game.make_move(&mv);
        score = score.max(-pure_negamax(game, depth - 1, side_to_move.opposite()));
        game.unmake_move(&mv);
    }

    score
}

pub fn pure_negamax_search(game: &mut ChessBoard, depth: i64, side_to_move: Color) -> (i64, Option<Move>) {
    let mut best_move = None;
    let mut best_score = i64::MIN;

    let moves = game.generate_moves(side_to_move);

    for mv in &moves {
        game.make_move(&mv);
        let score = -pure_negamax(game, depth - 1, side_to_move.opposite());
        game.unmake_move(&mv);
        if score >= best_score {
            best_move = Some(mv.clone());
            best_score = score;
        }
    }

    let perspective = if side_to_move == Color::White {1} else {-1};
    best_score = best_score * perspective;

    // Return best move found, or none
    (best_score, best_move)
}

fn minimax_alpha_beta(
    game: &mut ChessBoard,
    depth: u64,
    mut alpha: i64,
    mut beta: i64,
    side_to_move: Color
) -> i64 {
    // Terminal node check
    if depth == 0 {
        return game.evaluate();
    }

    let moves = game.generate_moves(side_to_move);

    if side_to_move == Color::White {
        let mut max_eval = i64::MIN;

        for mv in moves {
            game.make_move(&mv);
            let eval = minimax_alpha_beta(game, depth - 1, alpha, beta, side_to_move.opposite());
            game.unmake_move(&mv);

            max_eval = max_eval.max(eval);
            alpha = alpha.max(eval);

            if beta <= alpha {
                break; // Beta cutoff
            }
        }

        max_eval
    } else {
        let mut min_eval = i64::MAX;

        for mv in moves {
            game.make_move(&mv);
            let eval = minimax_alpha_beta(game, depth - 1, alpha, beta, side_to_move.opposite());
            game.unmake_move(&mv);

            min_eval = min_eval.min(eval);
            beta = beta.min(eval);

            if beta <= alpha {
                break; // Alpha cutoff
            }
        }

        min_eval
    }
}

pub fn minimax_alpha_beta_search(
    game: &mut ChessBoard,
    depth: u64,
    side_to_move: Color
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
        game.make_move(&mv);
        let score = minimax_alpha_beta(game, depth - 1, i64::MIN, i64::MAX, side_to_move.opposite());
        game.unmake_move(&mv);

        if side_to_move == Color::White {
            if score > best_score {
                best_score = score;
                best_move = Some(mv.clone());
                alpha = alpha.max(score)
            }
        } else {
            if score < best_score {
                best_score = score;
                best_move = Some(mv.clone());
                beta = beta.min(score);
            }
        }
    }

    (best_score, best_move)
}

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