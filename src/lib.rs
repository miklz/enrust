pub mod uci;
pub mod game_state;

pub fn start_engine() {
    uci::uci_main();
}