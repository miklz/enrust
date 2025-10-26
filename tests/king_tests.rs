#[cfg(test)]
mod king_tests {
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
    fn test_king_moves_center() {
        let mut game = setup_game_with_fen("8/8/8/3K4/8/8/8/8 w - - 0 1");

        let moves = game.generate_moves();
        assert_eq!(moves.len(), 8); // King in center has 8 moves
    }

    #[test]
    fn test_king_corner() {
        let mut game = setup_game_with_fen("K7/8/8/8/8/8/8/8 w - - 0 1");

        let moves = game.generate_moves();
        assert_eq!(moves.len(), 3); // King in corner has 3 moves
    }

    #[test]
    fn test_king_avoid_check() {
        let mut game = setup_game_with_fen("8/8/8/3K4/3r4/8/8/8 w - - 0 1");

        let moves = game.generate_moves();
        // King cannot move into check or stay in place
        // Should only have moves that escape the rook's attack
        assert_eq!(moves.len(), 5);
        assert!(moves.contains(&"d5d4".to_string())); // Capture attacker
        assert!(!moves.contains(&"d5d6".to_string())); // Stay in check
        assert!(!moves.contains(&"d5d5".to_string())); // Stay in place
    }
}
