mod move_generator {
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
    fn test_wrong_pawn_move() {
        // During test matches, invalid moves were generated in these positions
        let position_illegal_move: [(&str, &str); 3] = [
            (
                "1B1k1bQ1/5R2/3pp1Pp/8/P3P3/1P1P4/N1P2P2/R3KBN1 b - - 0 36",
                "g4g5",
            ),
            ("1r2k1nr/8/pp1b2p1/n4p1p/2P4P/8/K5b1/2q5 w k - 0 31", "c4d5"),
            (
                "2Q5/3Bk3/7P/p2Q1p2/8/P1PPp3/RP2PPKR/1N4N1 b - - 0 39",
                "g5g4",
            ),
        ];

        for (position, illegal_move) in position_illegal_move {
            let mut game = setup_game_with_fen(position);
            let moves = game.generate_moves();

            assert!(
                !moves.contains(&illegal_move.to_string()),
                "Ilegal move {} generated",
                illegal_move
            );
        }
    }
}
