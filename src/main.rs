use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    // If first argument is "bench", run benchmark mode
    if args.len() > 1 && args[1] == "bench" {
        enrust::run_benchmark();
    } else {
        // Normal engine operation (UCI)
        enrust::start_engine();
    }
}
