use simplelog::*;
use std::sync::Once;

pub fn initialize(debug: bool, other_crates: bool) {
    static START: Once = Once::new();

    START.call_once(move || {
        let level = if debug {
            LevelFilter::Debug
        } else {
            LevelFilter::Info
        };

        let my_crate_name = env!("CARGO_PKG_NAME").replace("-", "_");

        let mut loggers: Vec<Box<dyn SharedLogger>> = Vec::with_capacity(2);

        let mut config = ConfigBuilder::new();

        config.set_target_level(LevelFilter::Trace);
        config.set_thread_level(LevelFilter::Trace);

        if !other_crates {
            config.add_filter_allow(my_crate_name);
        }

        loggers.push(TermLogger::new(level, config.build(), TerminalMode::Mixed));

        CombinedLogger::init(loggers).unwrap();
    });
}
