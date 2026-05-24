use crate::search_options::SearchOptions;

pub struct EngineCommand {
    pub search_options: SearchOptions,
    pub stop: bool,
    pub quit: bool,
    pub ponderhit: bool,
}

impl EngineCommand {
    pub fn go(options: SearchOptions) -> EngineCommand {
        EngineCommand {
            search_options: options,
            stop: false,
            quit: false,
            ponderhit: false,
        }
    }

    pub fn stop() -> EngineCommand {
        EngineCommand {
            search_options: SearchOptions::default(),
            stop: true,
            quit: false,
            ponderhit: false,
        }
    }

    pub fn quit() -> EngineCommand {
        EngineCommand {
            search_options: SearchOptions::default(),
            stop: true,
            quit: true,
            ponderhit: false,
        }
    }

    pub fn ponderhit() -> EngineCommand {
        EngineCommand {
            search_options: SearchOptions::default(),
            stop: false,
            quit: false,
            ponderhit: true,
        }
    }
}
