#[cfg(test)]
mod rook_tests {
    use enrust::game_state::GameState;

    #[test]
    fn test_rook_moves_center() {
        let mut game = GameState::default();
        game.set_fen_position("8/8/8/3R4/8/8/8/8 w - - 0 1");

        let moves = game.generate_moves();
        assert_eq!(moves.len(), 14); // Rook in center has 14 moves
    }

    #[test]
    fn test_rook_vertical_horizontal() {
        let mut game = GameState::default();
        game.set_fen_position("8/8/3p4/2pRp3/3p4/8/8/8 w - - 0 1");

        let moves = game.generate_moves();
        // Should capture all 4 pawns (up, down, left, right)
        assert_eq!(moves.len(), 4);
    }

    #[test]
    fn test_rook_pin() {
        let mut game = GameState::default();
        game.set_fen_position("1r6/8/8/8/8/q7/R7/K7 w - - 0 1");

        let moves = game.generate_moves();
        // Rook is pinned to king by queen, it can only take the queen,
        // King cannot move.
        assert_eq!(moves.len(), 1);
        assert!(moves.contains(&"a2a3".to_string()));
    }
}
