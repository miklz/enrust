use std::time::Duration;
use std::time::Instant;

use divan::Bencher;
use enrust::game_state::GameState;

fn main() {
    divan::main();
}

#[divan::bench(
    args = [1, 2, 3, 4], // Different depths
)]
fn bench_perft_different_depths(bencher: Bencher, depth: u64) {
    let mut game = GameState::new(None);
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

    let mut game = GameState::new(None);

    game.set_fen_position(fen);

    bencher.bench_local(|| game.perft_debug(depth, false));
}

#[divan::bench(
    sample_count = 1,
    args = [
        ("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", 3, 5), // Initial position
        ("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1", 3, 4), // Kiwipete
        ("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1", 3, 4), // Position 3
    ],
)]
fn benchmark_perft_nps(params: (&str, u32, u64)) {
    let (fen, measured_runs, depth) = params;

    let mut game = GameState::new(None);
    game.set_fen_position(fen);

    let mut durations = Vec::new();
    let mut total_nodes = 0;

    for _run in 0..measured_runs {
        let start_time = Instant::now();
        let nodes = game.perft_debug(depth, false);
        let duration = start_time.elapsed();

        durations.push(duration);
        total_nodes = nodes; // Same for each run at same depth
    }

    // Calculate statistics
    let total_time: Duration = durations.iter().sum::<Duration>();
    let avg_duration: Duration = total_time / measured_runs;
    let nps = total_nodes as f64 / avg_duration.as_secs_f64();

    print!(
        "\n   ├─  Depth {}, Nodes searched: {:.0}, Nodes/second: {:.0}\t\t\t\t",
        depth, total_nodes, nps
    );
}
