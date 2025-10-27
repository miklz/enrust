use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use divan::{Bencher, black_box};
use enrust::game_state::ChessBoard;
use enrust::game_state::Color;
use enrust::game_state::GameState;
use enrust::game_state::board::search::{
    minimax_alpha_beta_search, pure_minimax_search, pure_negamax_search,
};
use enrust::game_state::{TranspositionTable, Zobrist};

fn main() {
    divan::main();
}

fn setup_game(fen: &str) -> ChessBoard {
    let zobrist_keys = Arc::new(Zobrist::new());
    let transposition_table = Arc::new(TranspositionTable::new(256));

    let mut game = GameState::new(zobrist_keys, transposition_table);
    assert!(game.set_fen_position(fen), "Failed to set FEN: {}", fen);
    game.get_chess_board().clone()
}

#[divan::bench(
args = [
    ("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"),
    ("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1"),
    ("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1"),
    ("rnbqkbnr/pp1ppppp/8/2p5/4P3/8/PPPP1PPP/RNBQKBNR w KQkq c6 0 1")
],
)]
fn bench_minimax(bencher: Bencher, fen: &str) {
    let mut game = setup_game(fen);

    let stop_flag = Arc::new(AtomicBool::new(false));
    bencher.bench_local(|| {
        let (score, _) = pure_minimax_search(&mut game, 4, Color::White, stop_flag.clone());
        black_box(score);
    });
}

#[divan::bench(
args = [
    ("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"),
    ("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1"),
    ("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1"),
    ("rnbqkbnr/pp1ppppp/8/2p5/4P3/8/PPPP1PPP/RNBQKBNR w KQkq c6 0 1")
],
)]
fn bench_negamax(bencher: Bencher, fen: &str) {
    let mut game = setup_game(fen);

    let stop_flag = Arc::new(AtomicBool::new(false));
    bencher.bench_local(|| {
        let (score, _) = pure_negamax_search(&mut game, 4, Color::White, stop_flag.clone());
        black_box(score);
    });
}

#[divan::bench(
args = [
    ("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"),
    ("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1"),
    ("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1"),
    ("rnbqkbnr/pp1ppppp/8/2p5/4P3/8/PPPP1PPP/RNBQKBNR w KQkq c6 0 1")
],
)]
fn bench_minimax_alpha_beta(bencher: Bencher, fen: &str) {
    let mut game = setup_game(fen);

    let stop_flag = Arc::new(AtomicBool::new(false));
    bencher.bench_local(|| {
        let (score, _) = minimax_alpha_beta_search(&mut game, 4, Color::White, stop_flag.clone());
        black_box(score);
    });
}
