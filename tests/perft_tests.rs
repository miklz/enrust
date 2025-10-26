#[cfg(test)]
mod perft_tests {
    use std::sync::Arc;

    use enrust::game_state::GameState;
    use enrust::game_state::{TranspositionTable, Zobrist};

    fn run_perft_test(fen: &str, depth: u64, expected_nodes: u64) {
        let zobrist_keys = Arc::new(Zobrist::new());

        let shared_transposition_table = Arc::new(TranspositionTable::new(256));

        let mut game = GameState::new(zobrist_keys, shared_transposition_table);
        assert!(game.set_fen_position(fen), "Failed to set FEN: {}", fen);

        let nodes = game.perft_debug(depth, false);
        assert_eq!(
            nodes, expected_nodes,
            "Perft({}) failed for FEN: {}\nExpected: {}, Got: {}",
            depth, fen, expected_nodes, nodes
        );
    }

    // Standard positions from chess programming wiki
    #[test]
    fn test_perft_initial_position() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        run_perft_test(fen, 1, 20);
        run_perft_test(fen, 2, 400);
        run_perft_test(fen, 3, 8902);
        run_perft_test(fen, 4, 197281);
        // run_perft_test(fen, 5, 4865609);  // Deep test for later
    }

    #[test]
    fn test_perft_kiwipete() {
        // Famous test position with many captures and checks
        let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
        run_perft_test(fen, 1, 48);
        run_perft_test(fen, 2, 2039);
        run_perft_test(fen, 3, 97862);
        // run_perft_test(fen, 4, 4085603);
    }

    #[test]
    fn test_perft_position_3() {
        // Position with en passant and pawn moves
        let fen = "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1";
        run_perft_test(fen, 1, 14);
        run_perft_test(fen, 2, 191);
        run_perft_test(fen, 3, 2812);
        run_perft_test(fen, 4, 43238);
        // run_perft_test(fen, 5, 674624);
    }

    #[test]
    fn test_perft_position_4() {
        // Position with promotions
        let fen = "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1";
        run_perft_test(fen, 1, 6);
        run_perft_test(fen, 2, 264);
        run_perft_test(fen, 3, 9467);
        // run_perft_test(fen, 4, 422333);
    }

    #[test]
    fn test_perft_position_5() {
        // Another complex position
        let fen = "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8";
        run_perft_test(fen, 1, 44);
        run_perft_test(fen, 2, 1486);
        run_perft_test(fen, 3, 62379);
        // run_perft_test(fen, 4, 2103487);
    }

    #[test]
    fn test_perft_castling() {
        // Position focused on castling
        let fen = "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1";
        run_perft_test(fen, 1, 26); // Includes castling moves
        run_perft_test(fen, 2, 568);
    }

    #[test]
    fn test_perft_promotion() {
        // Position from: http://www.rocechess.ch/perft.html
        // Discover promotion bugs
        let fen = "n1n5/PPPk4/8/8/8/8/4Kppp/5N1N b - - 0 1";
        run_perft_test(fen, 1, 24);
        run_perft_test(fen, 2, 496);
        run_perft_test(fen, 3, 9483);
    }
}
