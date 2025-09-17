use divan::Bencher;
use enrust::game_state::GameState;

fn main() {
    divan::main();
}

#[divan::bench(
    args = [1, 2, 3, 4], // Different depths
)]
fn bench_perft_different_depths(bencher: Bencher, depth: u64) {
    let mut game = GameState::default();
    game.set_fen_position("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");

    bencher.bench_local(|| game.perft_debug(depth, false));
}

#[divan::bench(
    args = [
        ("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", 3), // Initial position
        ("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1", 2), // Kiwipete
        ("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1", 3), // Position 3
    ],
)]
fn bench_perft_multiple_positions(bencher: Bencher, params: (&str, u64)) {
    let (fen, depth) = params;
    let mut game = GameState::default();
    game.set_fen_position(&fen);

    bencher.bench_local(|| game.perft_debug(depth, false));
}
