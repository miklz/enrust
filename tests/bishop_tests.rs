#[cfg(test)]
mod bishop_tests {
    use std::sync::{Arc, RwLock};

    use enrust::game_state::GameState;
    use enrust::game_state::{TranspositionTable, Zobrist};

    fn setup_game_with_fen(fen: &str) -> GameState {
        let zobrist_keys = Arc::new(Zobrist::new());

        let transposition_table = Arc::new(RwLock::new(TranspositionTable::new(256)));

        let mut game = GameState::new(zobrist_keys, transposition_table);
        game.set_fen_position(fen);
        game
    }

    #[test]
    fn test_bishop_moves_center() {
        let mut game = setup_game_with_fen("8/8/8/3B4/8/8/8/8 w - - 0 1");

        let moves = game.generate_moves();
        assert_eq!(moves.len(), 13); // Bishop in center has 13 moves
    }

    #[test]
    fn test_bishop_diagonal_captures() {
        let mut game = setup_game_with_fen("8/1p6/8/3B4/8/8/6p1/8 w - - 0 1");

        let moves = game.generate_moves();

        assert!(moves.contains(&"d5b7".to_string())); // Capture attacker
        assert!(moves.contains(&"d5g2".to_string())); // Capture attacker
        assert!(!moves.contains(&"d5h1".to_string())); // Can't go past attacker
    }

    #[test]
    fn test_bishop_color_bound() {
        let mut game = setup_game_with_fen("8/8/8/3B4/8/8/8/8 w - - 0 1");

        let moves = game.generate_moves();
        // Bishop should only move on light squares from d5
        let light_square_moves = moves
            .iter()
            .filter(|m| {
                let target = &m[2..4];
                // Light squares: a8, c6, e4, g2, etc.
                (target.chars().next().unwrap() as u8 - b'a' + target.chars().nth(1).unwrap() as u8
                    - b'1')
                    % 2
                    != 0
            })
            .count();

        assert_eq!(light_square_moves, moves.len()); // All moves should be on light squares
    }
}
