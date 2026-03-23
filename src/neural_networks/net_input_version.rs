#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetInputVersion {
    V1,
    V2,
}

impl NetInputVersion {
    pub fn from_planes(planes: usize) -> Result<NetInputVersion, String> {
        match planes {
            7 => Ok(NetInputVersion::V1),
            17 => Ok(NetInputVersion::V2),
            _ => Err(format!(
                "Unable to infer network input format from {planes} channels."
            )),
        }
    }

    pub fn planes(&self) -> usize {
        match self {
            NetInputVersion::V1 => 7,
            NetInputVersion::V2 => 17,
        }
    }
}
