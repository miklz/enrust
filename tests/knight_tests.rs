#[cfg(test)]
mod knight_tests {
    use enrust::game_state::GameState;

    #[test]
    fn test_knight_moves_center() {
        let mut game = GameState::default();
        game.set_fen_position("8/8/8/3N4/8/8/8/8 w - - 0 1");

        let moves = game.generate_moves();
        assert_eq!(moves.len(), 8); // Knight in center has 8 moves
    }

    #[test]
    fn test_knight_corner() {
        let mut game = GameState::default();
        game.set_fen_position("N7/8/8/8/8/8/8/8 w - - 0 1");

        let moves = game.generate_moves();
        assert_eq!(moves.len(), 2); // Knight in corner has 2 moves
    }

    #[test]
    fn test_knight_jump_over_pieces() {
        let mut game = GameState::default();
        game.set_fen_position("8/1PPP4/1PNP4/1PPP4/8/8/8/8 w - - 0 1");

        let moves = game.generate_moves();
        // Knight should be able to jump over surrounding pawns
        assert!(moves.len() > 0);
        assert!(moves.contains(&"c6a5".to_string()) || moves.contains(&"c6a7".to_string()));
    }

    #[test]
    fn test_knight_jump_over_enemy_pieces() {
        let mut game = GameState::default();
        game.set_fen_position("8/1ppp4/1pNp4/1ppp4/8/8/8/8 w - - 0 1");

        let moves = game.generate_moves();
        // Knight should be able to jump over surrounding pawns
        assert_eq!(moves.len(), 8);
        assert!(moves.contains(&"c6a5".to_string()) || moves.contains(&"c6a7".to_string()));
    }
}
