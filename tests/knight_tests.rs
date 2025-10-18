#[cfg(test)]
mod knight_tests {
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
    fn test_knight_moves_center() {
        let mut game = setup_game_with_fen("8/8/8/3N4/8/8/8/8 w - - 0 1");

        let moves = game.generate_moves();
        assert_eq!(moves.len(), 8); // Knight in center has 8 moves
    }

    #[test]
    fn test_knight_corner() {
        let mut game = setup_game_with_fen("N7/8/8/8/8/8/8/8 w - - 0 1");

        let moves = game.generate_moves();
        assert_eq!(moves.len(), 2); // Knight in corner has 2 moves
    }

    #[test]
    fn test_knight_jump_over_pieces() {
        let mut game = setup_game_with_fen("8/1PPP4/1PNP4/1PPP4/8/8/8/8 w - - 0 1");

        let moves = game.generate_moves();
        // Knight should be able to jump over surrounding pawns
        assert!(!moves.is_empty());
        assert!(moves.contains(&"c6a5".to_string()) || moves.contains(&"c6a7".to_string()));
    }

    #[test]
    fn test_knight_jump_over_enemy_pieces() {
        let mut game = setup_game_with_fen("8/1ppp4/1pNp4/1ppp4/8/8/8/8 w - - 0 1");

        let moves = game.generate_moves();
        // Knight should be able to jump over surrounding pawns
        assert_eq!(moves.len(), 8);
        assert!(moves.contains(&"c6a5".to_string()) || moves.contains(&"c6a7".to_string()));
    }
}
