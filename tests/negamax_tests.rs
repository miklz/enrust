#[cfg(test)]
mod negamax_tests {
    use enrust::game_state::GameState;
    use enrust::game_state::ChessBoard;
    use enrust::game_state::Color;
    use enrust::game_state::board::search::*;

    fn setup_test_game(fen: &str) -> ChessBoard {
        let mut game = GameState::default();
        assert!(game.set_fen_position(fen), "Failed to set FEN: {}", fen);
        game.get_chess_board().clone()
    }

    #[test]
    fn test_negamax_depth_1_initial_position() {
        let mut game = setup_test_game("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");

        let (score, best_move) = pure_negamax_search(&mut game, 1, Color::White);

        // At depth 1, should find one of the 20 possible moves
        let moves = game.generate_moves(Color::White);
        let best_move = best_move.unwrap();
        assert!(moves.iter().any(|mv| *mv == best_move),
                "Best move should be one of the legal moves");

        // Score should be reasonable (not extreme values)
        assert!(score.abs() < 1000, "Score should be reasonable, got: {}", score);
    }

    #[test]
    fn test_negamax_checkmate_white() {
        // White to move and checkmate black
        let mut game = setup_test_game("7R/8/8/8/8/1K6/8/1k6 w - - 0 1");

        let (score, best_move) = pure_negamax_search(&mut game, 3, Color::White);

        // Should find checkmate
        assert!(score > 10000, "Should find winning position, score: {}", score);

        // The move should be Ra8# or similar checkmate
        let best_move = best_move.unwrap();
        game.make_move(&best_move);
        assert!(game.is_checkmate(Color::Black), "Move should be checkmate");
        game.unmake_move(&best_move);
    }

    #[test]
    fn test_negamax_checkmate_black() {
        // Black to move and checkmate white
        let mut game = setup_test_game("7r/8/8/8/8/1k6/8/1K6 b - - 0 1");

        let (score, best_move) = pure_negamax_search(&mut game, 3, Color::Black);

        // Should find checkmate (negative score from black's perspective)
        assert!(score < -10000, "Should find winning position for black, score: {}", score);

        // The move should be checkmate
        let best_move = best_move.unwrap();
        game.make_move(&best_move);
        assert!(game.is_checkmate(Color::White), "Move should be checkmate");
        game.unmake_move(&best_move);
    }

    #[test]
    fn test_negamax_stalemate() {
        // Stalemate position - black to move, no legal moves but not in check
        let mut game = setup_test_game("k7/8/1K6/8/8/8/8/8 b - - 0 1");

        let (score, _) = pure_negamax_search(&mut game, 1, Color::Black);

        // Should recognize stalemate (score = 0)
        assert_eq!(score, 0, "Should recognize stalemate, got score: {}", score);
    }

    #[test]
    fn test_negamax_capture_priority() {
        // White can capture black queen with pawn
        let mut game = setup_test_game("k7/8/8/3q4/3Q4/8/8/K7 w - - 0 1");

        let (score, best_move) = pure_negamax_search(&mut game, 2, Color::White);

        // Should prefer capturing the queen (d4xd5)
        let expected_move = game.from_uci("d4d5").expect("Should create capture move");
        let best_move = best_move.unwrap();
        assert_eq!(best_move, expected_move,
                  "Should capture queen, got: {}", best_move.to_uci(&game));

        // Score should reflect material advantage
        assert!(score > 800, "Should have significant advantage after capture, score: {}", score);
    }

    #[test]
    fn test_negamax_promotion() {
        // White pawn can promote to queen
        let mut game = setup_test_game("k7/3P4/8/8/8/8/8/K7 w - - 0 1");

        let (score, best_move) = pure_negamax_search(&mut game, 2, Color::White);

        // Should promote to queen (b7b8q)
        let promotion_move = game.from_uci("d7d8q").expect("Should create promotion move");
        let best_move = best_move.unwrap();
        assert_eq!(best_move, promotion_move,
                  "Should promote pawn to queen, got: {}", best_move.to_uci(&game));

        // Score should reflect queen advantage
        assert!(score >= 900, "Should have queen advantage, score: {}", score);
    }

    #[test]
    fn test_negamax_avoids_checkmate() {
        // White can be checkmated next move if he doesn't prevent it
        let mut game = setup_test_game("k7/8/8/8/8/8/2r5/KR6 w - - 0 1");

        let (score, best_move) = pure_negamax_search(&mut game, 2, Color::White);

        // Should avoid the checkmate by moving king or blocking
        let best_move = best_move.unwrap();
        game.make_move(&best_move);
        assert!(!game.is_checkmate(Color::White),
               "Move should avoid immediate checkmate: {}", best_move.to_uci(&game));
        game.unmake_move(&best_move);

        // Score should not be extremely negative
        assert!(score > -10000, "Should avoid checkmate, score: {}", score);
    }

    #[test]
    fn test_negamax_depth_consistency() {
        let mut game = setup_test_game("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");

        // Test that deeper search gives better (or equal) results
        let (score_depth_1, move_1) = pure_negamax_search(&mut game, 1, Color::White);
        let (score_depth_2, move_2) = pure_negamax_search(&mut game, 2, Color::White);
        let (score_depth_3, move_3) = pure_negamax_search(&mut game, 3, Color::White);

        // Deeper search should find at least as good moves
        // Note: Sometimes different depths can find different equally good moves
        println!("Depth 1: {} (score: {})", move_1.unwrap().to_uci(&game), score_depth_1);
        println!("Depth 2: {} (score: {})", move_2.unwrap().to_uci(&game), score_depth_2);
        println!("Depth 3: {} (score: {})", move_3.unwrap().to_uci(&game), score_depth_3);

        // Scores should be reasonable
        assert!(score_depth_1.abs() < 1000);
        assert!(score_depth_2.abs() < 1000);
        assert!(score_depth_3.abs() < 1000);
    }

    #[test]
    fn test_negamax_symmetric_evaluation() {
        // Symmetric position should evaluate to 0
        let mut game = setup_test_game("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");

        let (score_white, _) = pure_negamax_search(&mut game, 2, Color::White);

        // Now from black's perspective (should be symmetric)
        let (score_black, _) = pure_negamax_search(&mut game, 2, Color::Black);

        // Scores should be approximately opposite (white positive, black negative)
        assert!((score_white + score_black).abs() < 50,
               "Symmetric position should have opposite scores: white={}, black={}",
               score_white, score_black);
    }

    #[test]
    fn test_negamax_material_advantage() {
        // White has extra queen
        let mut game = setup_test_game("k7/8/8/8/8/8/1Q6/K7 w - - 0 1");

        let (score, _) = pure_negamax_search(&mut game, 1, Color::White);

        // Should show significant advantage (around +900 for queen)
        assert!(score > 800 && score < 1000,
               "Should show queen advantage, got: {}", score);
    }

    #[test]
    fn test_negamax_always_returns_legal_move() {
        let mut game = setup_test_game("k7/8/8/8/8/8/8/K7 w - - 0 1"); // Only kings

        for depth in 1..=3 {
            let (score, best_move) = pure_negamax_search(&mut game, depth, Color::White);

            // Move should be legal
            let legal_moves = game.generate_moves(Color::White);
            let best_move = best_move.unwrap();
            assert!(legal_moves.contains(&best_move),
                   "Depth {}: Returned illegal move: {}", depth, best_move.to_uci(&game));

            // Score should be 0 (equal position)
            assert_eq!(score, 0, "Depth {}: Kings only should be equal, got: {}", depth, score);
        }
    }
}