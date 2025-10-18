#[cfg(test)]
mod rook_tests {
    use std::sync::{Arc, RwLock};

    use enrust::game_state::GameState;
    use enrust::game_state::{TranspositionTable, Zobrist};

    fn setup_game_with_fen(fen: &str) -> GameState {
        let zobrist_keys = Arc::new(Zobrist::new());

        let shared_transposition_table = Arc::new(RwLock::new(TranspositionTable::new(256)));

        let mut game = GameState::new(zobrist_keys, shared_transposition_table);
        game.set_fen_position(fen);
        game
    }

    #[test]
    fn test_rook_moves_center() {
        let mut game = setup_game_with_fen("8/8/8/3R4/8/8/8/8 w - - 0 1");

        let moves = game.generate_moves();
        assert_eq!(moves.len(), 14); // Rook in center has 14 moves
    }

    #[test]
    fn test_rook_vertical_horizontal() {
        let mut game = setup_game_with_fen("8/8/3p4/2pRp3/3p4/8/8/8 w - - 0 1");

        let moves = game.generate_moves();
        // Should capture all 4 pawns (up, down, left, right)
        assert_eq!(moves.len(), 4);
    }

    #[test]
    fn test_rook_pin() {
        let mut game = setup_game_with_fen("1r6/8/8/8/8/q7/R7/K7 w - - 0 1");

        let moves = game.generate_moves();
        // Rook is pinned to king by queen, it can only take the queen,
        // King cannot move.
        assert_eq!(moves.len(), 1);
        assert!(moves.contains(&"a2a3".to_string()));
    }
}
