use crate::search_options::SearchOptions;

pub struct EngineCommand {
    pub search_options: SearchOptions,
    pub stop: bool,
    pub quit: bool,
    pub bench: bool,
    pub bench_depth: u32,
}

impl EngineCommand {
    pub fn go(options: SearchOptions) -> EngineCommand {
        EngineCommand {
            search_options: options,
            stop: false,
            quit: false,
            bench: false,
            bench_depth: 0,
        }
    }

    pub fn stop() -> EngineCommand {
        EngineCommand {
            search_options: SearchOptions::default(),
            stop: true,
            quit: false,
            bench: false,
            bench_depth: 0,
        }
    }

    pub fn quit() -> EngineCommand {
        EngineCommand {
            search_options: SearchOptions::default(),
            stop: true,
            quit: true,
            bench: false,
            bench_depth: 0,
        }
    }

    pub fn bench(depth: u32) -> EngineCommand {
        EngineCommand {
            search_options: SearchOptions::default(),
            stop: false,
            quit: false,
            bench: true,
            bench_depth: depth,
        }
    }
}
