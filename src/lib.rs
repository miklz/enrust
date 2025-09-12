pub mod game_state;
pub mod uci;

pub fn start_engine() {
    uci::uci_main();
}
