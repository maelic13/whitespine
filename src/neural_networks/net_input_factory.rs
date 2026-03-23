use chess::Board;

use super::NetInputVersion;
use super::net_input_v1;
use super::net_input_v2;

pub type NetInputFn = fn(&Board) -> Vec<f32>;

pub struct NetInputFactory;

impl NetInputFactory {
    pub fn from_version(version: NetInputVersion) -> NetInputFn {
        match version {
            NetInputVersion::V1 => net_input_v1::board_to_input,
            NetInputVersion::V2 => net_input_v2::board_to_input,
        }
    }
}
