#[cfg(test)]
mod queen_tests {
    use enrust::game_state::GameState;

    #[test]
    fn test_queen_moves_center() {
        let mut game = GameState::default();
        game.set_fen_position("8/8/8/3Q4/8/8/8/8 w - - 0 1");

        let moves = game.generate_moves();
        assert_eq!(moves.len(), 27); // Queen in center has 27 moves
        assert!(moves.contains(&"d5d6".to_string()));
        assert!(moves.contains(&"d5d4".to_string()));
        assert!(moves.contains(&"d5e5".to_string()));
        assert!(moves.contains(&"d5c5".to_string()));
        assert!(moves.contains(&"d5e6".to_string()));
        assert!(moves.contains(&"d5c4".to_string()));
    }

    #[test]
    fn test_queen_captures() {
        let mut game = GameState::default();
        game.set_fen_position("8/8/2ppp3/2pQp3/2ppp3/8/8/8 w - - 0 1");

        let moves = game.generate_moves();
        // Should have 8 captures (all diagonals and orthogonals)
        assert_eq!(moves.len(), 8);
    }

    #[test]
    fn test_queen_blocked() {
        let mut game = GameState::default();
        game.set_fen_position("8/8/2PPP3/2PQP3/2PPP3/8/8/8 w - - 0 1");

        let moves = game.generate_moves();
        let queen_moves = moves.iter().filter(|m| m.contains("d5")).count();
        assert_eq!(queen_moves, 0); // Completely surrounded by own pieces
    }
}
