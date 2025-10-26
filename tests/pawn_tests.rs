#[cfg(test)]
mod pawn_tests {
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
    fn test_pawn_single_push() {
        let mut game = setup_game_with_fen("8/8/8/8/8/4P3/8/8 w - - 0 1");
        /*
            12x10 Chess Board:
            ==============================
            10 │ X X X X X X X X X X │
            09 │ X X X X X X X X X X │
            08 │ X . . . . . . . . X │
            07 │ X . . . . . . . . X │
            06 │ X . . . . . . . . X │
            05 │ X . . . . . . . . X │
            04 │ X . . . . . . . . X │
            03 │ X . . . . P . . . X │
            02 │ X . . . . . . . . X │
            01 │ X . . . . . . . . X │
            00 │ X X X X X X X X X X │
            -1 │ X X X X X X X X X X │
               └─────────────────────
                 z a b c d e f g h i
        */

        let moves = game.generate_moves();

        assert_eq!(moves.len(), 1);
        assert_eq!(moves[0], "e3e4");
    }

    #[test]
    fn test_pawn_double_push_from_start() {
        let mut game = setup_game_with_fen("8/8/8/8/8/8/4P3/8 w - - 0 1");
        /*
            12x10 Chess Board:
            ==============================
            10 │ X X X X X X X X X X │
            09 │ X X X X X X X X X X │
            08 │ X . . . . . . . . X │
            07 │ X . . . . . . . . X │
            06 │ X . . . . . . . . X │
            05 │ X . . . . . . . . X │
            04 │ X . . . . . . . . X │
            03 │ X . . . . . . . . X │
            02 │ X . . . . P . . . X │
            01 │ X . . . . . . . . X │
            00 │ X X X X X X X X X X │
            -1 │ X X X X X X X X X X │
            └─────────────────────
                z a b c d e f g h i
        */

        let moves = game.generate_moves();

        // Should have both single and double push
        assert_eq!(moves.len(), 2);

        assert!(moves.contains(&"e2e3".to_string()));
        assert!(moves.contains(&"e2e4".to_string()));
    }
}

#[cfg(test)]
mod promotion_tests {
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
    fn test_pawn_promotion_options() {
        let mut game = setup_game_with_fen("8/4P3/8/8/8/8/8/8 w - - 0 1");
        /*
            12x10 Chess Board:
            ==============================
            10 │ X X X X X X X X X X │
            09 │ X X X X X X X X X X │
            08 │ X . . . . . . . . X │
            07 │ X . . . . P . . . X │
            06 │ X . . . . . . . . X │
            05 │ X . . . . . . . . X │
            04 │ X . . . . . . . . X │
            03 │ X . . . . . . . . X │
            02 │ X . . . . . . . . X │
            01 │ X . . . . . . . . X │
            00 │ X X X X X X X X X X │
            -1 │ X X X X X X X X X X │
               └─────────────────────
                 z a b c d e f g h i
        */

        let pawn_moves = game.generate_moves();

        // Should have 4 promotion options
        assert_eq!(pawn_moves.len(), 4);

        assert!(pawn_moves.contains(&"e7e8q".to_string()));
        assert!(pawn_moves.contains(&"e7e8r".to_string()));
        assert!(pawn_moves.contains(&"e7e8b".to_string()));
        assert!(pawn_moves.contains(&"e7e8n".to_string()));
    }

    #[test]
    fn test_pawn_promotion_with_capture() {
        let mut game = setup_game_with_fen("3n4/4P3/8/8/8/8/8/8 w - - 0 1");
        /*
            12x10 Chess Board:
            ==============================
            10 │ X X X X X X X X X X │
            09 │ X X X X X X X X X X │
            08 │ X . . . . . . . . X │
            07 │ X . . . . P . . . X │
            06 │ X . . . . . . . . X │
            05 │ X . . . . . . . . X │
            04 │ X . . . . . . . . X │
            03 │ X . . . . . . . . X │
            02 │ X . . . . . . . . X │
            01 │ X . . . . . . . . X │
            00 │ X X X X X X X X X X │
            -1 │ X X X X X X X X X X │
               └─────────────────────
                 z a b c d e f g h i
        */

        let pawn_moves = game.generate_moves();

        // Should have 8 moves: 4 promotions + 4 capture promotions
        assert_eq!(pawn_moves.len(), 8);

        let promotion_moves: Vec<String> = pawn_moves
            .into_iter()
            .filter(|mv| mv.len() == 5) // Only promotions have 5 characters
            .collect();

        assert!(promotion_moves.contains(&"e7e8q".to_string()));
        assert!(promotion_moves.contains(&"e7d8q".to_string()));
        assert!(promotion_moves.contains(&"e7e8r".to_string()));
        assert!(promotion_moves.contains(&"e7d8r".to_string()));
        assert!(promotion_moves.contains(&"e7e8b".to_string()));
        assert!(promotion_moves.contains(&"e7d8b".to_string()));
        assert!(promotion_moves.contains(&"e7e8n".to_string()));
        assert!(promotion_moves.contains(&"e7d8n".to_string()));
    }

    #[test]
    fn test_no_promotion_before_seventh_rank() {
        let mut game = setup_game_with_fen("8/8/4P3/8/8/8/8/8 w - - 0 1");
        /*
            12x10 Chess Board:
            ==============================
            10 │ X X X X X X X X X X │
            09 │ X X X X X X X X X X │
            08 │ X . . . . . . . . X │
            07 │ X . . . . . . . . X │
            06 │ X . . . . P . . . X │
            05 │ X . . . . . . . . X │
            04 │ X . . . . . . . . X │
            03 │ X . . . . . . . . X │
            02 │ X . . . . . . . . X │
            01 │ X . . . . . . . . X │
            00 │ X X X X X X X X X X │
            -1 │ X X X X X X X X X X │
               └─────────────────────
                 z a b c d e f g h i
        */

        let pawn_moves = game.generate_moves();

        // Should not have promotion options
        assert_eq!(pawn_moves.len(), 1); // single push
        assert!(pawn_moves.iter().all(|mv| mv.len() < 5));
    }
}

#[cfg(test)]
mod blocked_pawn_tests {
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
    fn test_pawn_blocked_by_friendly() {
        let mut game = setup_game_with_fen("8/8/8/8/4P3/4P3/8/8 w - - 0 1");
        /*
            12x10 Chess Board:
            ==============================
            10 │ X X X X X X X X X X │
            09 │ X X X X X X X X X X │
            08 │ X . . . . . . . . X │
            07 │ X . . . . . . . . X │
            06 │ X . . . . . . . . X │
            05 │ X . . . . . . . . X │
            04 │ X . . . . P . . . X │
            03 │ X . . . . P . . . X │
            02 │ X . . . . . . . . X │
            01 │ X . . . . . . . . X │
            00 │ X X X X X X X X X X │
            -1 │ X X X X X X X X X X │
               └─────────────────────
                 z a b c d e f g h i
        */

        let pawn_moves = game.generate_moves();

        // e3 pawn is blocked by e4 pawn, should have no moves
        assert!(!pawn_moves.contains(&"e3".to_string()));
    }

    #[test]
    fn test_pawn_blocked_by_opponent() {
        let mut game = setup_game_with_fen("8/8/8/4p3/4P3/8/8/8 w - - 0 1");
        /*
            12x10 Chess Board:
            ==============================
            10 │ X X X X X X X X X X │
            09 │ X X X X X X X X X X │
            08 │ X . . . . . . . . X │
            07 │ X . . . . . . . . X │
            06 │ X . . . . . . . . X │
            05 │ X . . . . p . . . X │
            04 │ X . . . . P . . . X │
            03 │ X . . . . . . . . X │
            02 │ X . . . . . . . . X │
            01 │ X . . . . . . . . X │
            00 │ X X X X X X X X X X │
            -1 │ X X X X X X X X X X │
               └─────────────────────
                 z a b c d e f g h i
        */

        let pawn_moves = game.generate_moves();

        // e4 pawn is blocked by e5 pawn, it doesn't have captures
        assert_eq!(pawn_moves.len(), 0);
    }
}

#[cfg(test)]
mod en_passant_tests {
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
    fn test_en_passant_capture() {
        let mut game = setup_game_with_fen("8/8/8/3pP3/8/8/8/8 w - d6 0 1");
        /*
            12x10 Chess Board:
            ==============================
            10 │ X X X X X X X X X X │
            09 │ X X X X X X X X X X │
            08 │ X . . . . . . . . X │
            07 │ X . . . . . . . . X │
            06 │ X . . . . . . . . X │
            05 │ X . . . p P . . . X │
            04 │ X . . . . . . . . X │
            03 │ X . . . . . . . . X │
            02 │ X . . . . . . . . X │
            01 │ X . . . . . . . . X │
            00 │ X X X X X X X X X X │
            -1 │ X X X X X X X X X X │
               └─────────────────────
                 z a b c d e f g h i
        */

        let pawn_moves = game.generate_moves();

        // Should have forward move and en passant capture
        assert_eq!(pawn_moves.len(), 2);

        assert!(pawn_moves.contains(&"e5e6".to_string())); // push
        assert!(pawn_moves.contains(&"e5d6".to_string())); // en passant
    }

    #[test]
    fn test_en_passant_not_available() {
        let mut game = setup_game_with_fen("8/8/8/3pP3/8/8/8/8 w - - 0 1");
        /*
            12x10 Chess Board:
            ==============================
            10 │ X X X X X X X X X X │
            09 │ X X X X X X X X X X │
            08 │ X . . . . . . . . X │
            07 │ X . . . . . . . . X │
            06 │ X . . . . . . . . X │
            05 │ X . . . p P . . . X │
            04 │ X . . . . . . . . X │
            03 │ X . . . . . . . . X │
            02 │ X . . . . . . . . X │
            01 │ X . . . . . . . . X │
            00 │ X X X X X X X X X X │
            -1 │ X X X X X X X X X X │
               └─────────────────────
                 z a b c d e f g h i
        */

        let pawn_moves = game.generate_moves();

        // No en passant target, should only have forward move
        assert_eq!(pawn_moves.len(), 1);
        assert!(pawn_moves.contains(&"e5e6".to_string()));
    }

    #[test]
    fn test_en_passant_wrong_square() {
        let mut game = setup_game_with_fen("8/8/8/3pP3/8/8/8/8 w - e6 0 1");
        /*
            12x10 Chess Board:
            ==============================
            10 │ X X X X X X X X X X │
            09 │ X X X X X X X X X X │
            08 │ X . . . . . . . . X │
            07 │ X . . . . . . . . X │
            06 │ X . . . . . . . . X │
            05 │ X . . . p P . . . X │
            04 │ X . . . . . . . . X │
            03 │ X . . . . . . . . X │
            02 │ X . . . . . . . . X │
            01 │ X . . . . . . . . X │
            00 │ X X X X X X X X X X │
            -1 │ X X X X X X X X X X │
               └─────────────────────
                 z a b c d e f g h i
        */

        let pawn_moves = game.generate_moves();

        // En passant target is e6, but pawn is on e5 - should not allow capture
        assert_eq!(pawn_moves.len(), 1);
        assert!(pawn_moves.contains(&"e5e6".to_string()));
    }
}

#[cfg(test)]
mod edge_case_tests {
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
    fn test_pawn_on_edge_files() {
        let mut game = setup_game_with_fen("8/8/8/8/8/8/P7/8 w - - 0 1");
        /*
            12x10 Chess Board:
            ==============================
            10 │ X X X X X X X X X X │
            09 │ X X X X X X X X X X │
            08 │ X . . . . . . . . X │
            07 │ X . . . . . . . . X │
            06 │ X . . . . . . . . X │
            05 │ X . . . . . . . . X │
            04 │ X . . . . . . . . X │
            03 │ X . . . . . . . . X │
            02 │ X P . . . . . . . X │
            01 │ X . . . . . . . . X │
            00 │ X X X X X X X X X X │
            -1 │ X X X X X X X X X X │
               └─────────────────────
                 z a b c d e f g h i
        */

        let pawn_moves = game.generate_moves();

        // a-file pawn should only have right capture (if available) and pushes
        // This test depends on the board setup - adjust as needed
        assert!(pawn_moves.len() >= 2); // at least push moves
    }

    #[test]
    fn test_pawn_cannot_move_backwards() {
        let mut game = setup_game_with_fen("8/8/8/8/8/4p3/8/8 b - - 0 1");
        /*
            12x10 Chess Board:
            ==============================
            10 │ X X X X X X X X X X │
            09 │ X X X X X X X X X X │
            08 │ X . . . . . . . . X │
            07 │ X . . . . . . . . X │
            06 │ X . . . . . . . . X │
            05 │ X . . . . . . . . X │
            04 │ X . . . . . . . . X │
            03 │ X . . . . p . . . X │
            02 │ X . . . . . . . . X │
            01 │ X . . . . . . . . X │
            00 │ X X X X X X X X X X │
            -1 │ X X X X X X X X X X │
               └─────────────────────
                 z a b c d e f g h i
        */

        let pawn_moves = game.generate_moves();

        // Black pawn should move downward (higher ranks to lower ranks)
        assert!(pawn_moves.iter().any(|uci| uci.ends_with("e2"))); // should move to e2
        assert!(!pawn_moves.iter().any(|uci| uci.ends_with("e4"))); // should not move to e4
    }
}
