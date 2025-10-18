use std::sync::{Arc, RwLock};

use divan::Bencher;
use enrust::game_state::{GameState, TranspositionTable, Zobrist};

fn main() {
    divan::main();
}

#[divan::bench(
    args = [
        ("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"), // Initial position
        ("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1"), // Kiwipete
        ("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1"), // Position 3
        ("rnbqkbnr/pp1ppppp/8/2p5/4P3/8/PPPP1PPP/RNBQKBNR w KQkq c6 0 1") // e4 c5 have en passant
    ],
)]
fn setup_position(bencher: Bencher, fen: &str) {
    let zobrist_keys = Arc::new(Zobrist::new());
    let transposition_table = Arc::new(RwLock::new(TranspositionTable::new(256)));

    let mut game = GameState::new(zobrist_keys, transposition_table);

    bencher.bench_local(|| game.set_fen_position(fen))
}

#[divan::bench(
    args = [
        ("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"),
        ("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq - 0 1"),
        ("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1"),
        ("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R b KQkq - 0 1"),
        ("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1"),
        ("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 b - - 0 1"),
        ("rnbqkbnr/pp1ppppp/8/2p5/4P3/8/PPPP1PPP/RNBQKBNR w KQkq c6 0 1"),
        ("rnbqkbnr/pp1ppppp/8/2p5/4P3/8/PPPP1PPP/RNBQKBNR b KQkq c6 0 1")
    ],
)]
fn generate_moves(bencher: Bencher, fen: &str) {
    let zobrist_keys = Arc::new(Zobrist::new());
    let transposition_table = Arc::new(RwLock::new(TranspositionTable::new(256)));

    let mut game = GameState::new(zobrist_keys, transposition_table);
    game.set_fen_position(fen);

    bencher.bench_local(|| {
        game.generate_moves();
    });
}

#[divan::bench]
fn make_unmake_move(bencher: Bencher) {
    let zobrist_keys = Arc::new(Zobrist::new());
    let transposition_table = Arc::new(RwLock::new(TranspositionTable::new(256)));

    let mut game = GameState::new(zobrist_keys, transposition_table);
    game.set_fen_position("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 b - - 0 1");

    let moves: Vec<String> = game.generate_moves();

    bencher.bench_local(|| {
        for mv in &moves {
            game.make_move(mv.as_str());
            game.unmake_move(mv.as_str());
        }
    });
}
