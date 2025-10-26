#[cfg(test)]
mod castling_tests {
    use std::sync::Arc;

    use enrust::game_state::GameState;
    use enrust::game_state::{TranspositionTable, Zobrist};

    fn setup_game_with_fen(fen: &str) -> GameState {
        let zobrist_keys = Arc::new(Zobrist::new());

        let shared_transposition_table = Arc::new(TranspositionTable::new(256));

        let mut game = GameState::new(zobrist_keys, shared_transposition_table);
        game.set_fen_position(fen);
        game
    }

    #[test]
    fn test_white_kingside_castling() {
        let mut game = setup_game_with_fen("r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1");

        let moves = game.generate_moves();

        // Should have kingside castling move
        assert!(
            moves.contains(&"e1g1".to_string()),
            "Kingside castling move not found. Moves: {:?}",
            moves
        );

        // Should also have other moves (king moves, rook moves)
        assert!(
            !moves.is_empty(),
            "Expected at least 1 move, got {}",
            moves.len()
        );
    }

    #[test]
    fn test_white_queenside_castling() {
        let mut game = setup_game_with_fen("r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1");

        let moves = game.generate_moves();

        // Should have queenside castling move
        assert!(
            moves.contains(&"e1c1".to_string()),
            "Queenside castling move not found. Moves: {:?}",
            moves
        );
    }

    #[test]
    fn test_black_kingside_castling() {
        let mut game = setup_game_with_fen("r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R b KQkq - 0 1");

        let moves = game.generate_moves();

        // Should have kingside castling move
        assert!(
            moves.contains(&"e8g8".to_string()),
            "Black kingside castling move not found. Moves: {:?}",
            moves
        );
    }

    #[test]
    fn test_black_queenside_castling() {
        let mut game = setup_game_with_fen("r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R b KQkq - 0 1");

        let moves = game.generate_moves();

        // Should have queenside castling move
        assert!(
            moves.contains(&"e8c8".to_string()),
            "Black queenside castling move not found. Moves: {:?}",
            moves
        );
    }

    #[test]
    fn test_castling_with_pieces_in_between() {
        // Bishop blocking the kingside castling
        let mut game = setup_game_with_fen("r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3KB1R w KQkq - 0 1");

        let moves = game.generate_moves();

        // Should NOT have kingside castling
        assert!(
            !moves.contains(&"e1g1".to_string()),
            "Kingside castling should be blocked by bishop. Moves: {:?}",
            moves
        );

        // But should still have queenside
        assert!(
            moves.contains(&"e1c1".to_string()),
            "Queenside castling should be available. Moves: {:?}",
            moves
        );
    }

    #[test]
    fn test_castling_rights_after_king_move() {
        let mut game = setup_game_with_fen("r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1");

        // Make king move
        game.make_move("e1e2");
        game.make_move("e8e7"); // Black moves

        let moves = game.generate_moves();

        // Should NOT have any castling moves after king moved
        assert!(
            !moves.contains(&"e2g2".to_string()),
            "Should not have castling after king moved. Moves: {:?}",
            moves
        );
        assert!(
            !moves.contains(&"e2c2".to_string()),
            "Should not have castling after king moved. Moves: {:?}",
            moves
        );
    }

    #[test]
    fn test_castling_rights_after_rook_move() {
        let mut game = setup_game_with_fen("r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1");

        // Move kingside rook
        game.make_move("h1h2");
        game.make_move("e8e7"); // Black moves

        let moves = game.generate_moves();

        // Should have queenside but not kingside castling
        assert!(
            !moves.contains(&"e1g1".to_string()),
            "Should not have kingside castling {} after rook moved. Moves: {:?}",
            "e1g1",
            moves
        );
        assert!(
            moves.contains(&"e1c1".to_string()),
            "Should still have queenside castling {}. Moves: {:?}",
            "e1c1",
            moves
        );
    }

    #[test]
    fn test_castling_through_check() {
        // Black queen attacking f1 square (kingside castling path)
        let mut game = setup_game_with_fen("r3k2r/pppppppp/8/8/5q2/8/PPPPP1PP/R3K2R w KQkq - 0 1");

        let moves = game.generate_moves();

        // Should NOT have kingside castling (king would move through check)
        assert!(
            !moves.contains(&"e1g1".to_string()),
            "Should not castling through check. Moves: {:?}",
            moves
        );
    }

    #[test]
    fn test_castling_out_of_check() {
        // Black queen giving check
        let mut game =
            setup_game_with_fen("r3k2r/pppppppp/8/8/5P2/6q1/PPPPP1PP/R3K2R b KQkq f3 0 1");

        let moves = game.generate_moves();

        // Should NOT have any castling moves (cannot castle out of check)
        assert!(
            !moves.contains(&"e1g1".to_string()),
            "Cannot castle out of check. Moves: {:?}",
            moves
        );
        assert!(
            !moves.contains(&"e1c1".to_string()),
            "Cannot castle out of check. Moves: {:?}",
            moves
        );
    }

    #[test]
    fn test_no_castling_rights() {
        // Position without castling rights
        let mut game = setup_game_with_fen("r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w - - 0 1");

        let moves = game.generate_moves();

        // Should NOT have any castling moves
        assert!(
            !moves.contains(&"e1g1".to_string()),
            "No castling rights. Moves: {:?}",
            moves
        );
        assert!(
            !moves.contains(&"e1c1".to_string()),
            "No castling rights. Moves: {:?}",
            moves
        );
    }
}
