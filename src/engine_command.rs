use crate::search_options::SearchOptions;

pub struct EngineCommand {
    pub search_options: SearchOptions,
    pub stop: bool,
    pub quit: bool,
}

impl EngineCommand {
    pub fn default() -> EngineCommand {
        EngineCommand {
            search_options: SearchOptions::default(),
            stop: false,
            quit: false,
        }
    }

    pub fn go(options: SearchOptions) -> EngineCommand {
        EngineCommand {
            search_options: options,
            stop: false,
            quit: false,
        }
    }

    pub fn stop() -> EngineCommand {
        EngineCommand {
            search_options: SearchOptions::default(),
            stop: true,
            quit: false,
        }
    }

    pub fn quit() -> EngineCommand {
        EngineCommand {
            search_options: SearchOptions::default(),
            stop: true,
            quit: true,
        }
    }
}
