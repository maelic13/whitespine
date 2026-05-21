use crate::board::Color;
use crate::search_options::{EngineOptions, SearchLimits};

#[derive(Copy, Clone)]
pub(crate) struct RuntimeLimits {
    pub depth: usize,
    pub nodes: u64,
    pub soft_ms: f64,
    pub hard_ms: f64,
}

pub(crate) fn compute_runtime_limits(
    options: SearchLimits,
    engine_options: EngineOptions,
    side_to_move: Color,
    max_depth: usize,
) -> RuntimeLimits {
    let depth = if options.depth.is_finite() {
        options.depth.max(1.0) as usize
    } else {
        max_depth
    };
    let mut soft_ms = f64::INFINITY;
    let mut hard_ms = f64::INFINITY;

    if options.move_time > 0 {
        let available = (options.move_time as f64 - engine_options.move_overhead).max(1.0);
        soft_ms = available;
        hard_ms = available;
    } else {
        let (time, increment) = match side_to_move {
            Color::White => (options.white_time, options.white_increment),
            Color::Black => (options.black_time, options.black_increment),
        };
        if time > 0 {
            let remaining = (time as f64 - engine_options.move_overhead).max(1.0);
            let moves_to_go = if options.movestogo > 0 {
                options.movestogo as f64
            } else {
                30.0
            };
            soft_ms = (remaining / moves_to_go + increment as f64 * 0.75).max(1.0);
            hard_ms = (soft_ms * 4.0).min(remaining * 0.8).max(soft_ms);
        }
    }

    RuntimeLimits {
        depth,
        nodes: options.nodes,
        soft_ms,
        hard_ms,
    }
}
