#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeuristicType {
    Classical,
    NeuralNetwork,
}

impl HeuristicType {
    pub fn as_str(&self) -> &'static str {
        match self {
            HeuristicType::Classical => "classical",
            HeuristicType::NeuralNetwork => "neural_network",
        }
    }

    pub fn from_str(string: &str) -> Result<HeuristicType, String> {
        match string.trim().to_lowercase().as_str() {
            "classical" => Ok(HeuristicType::Classical),
            "neural_network" => Ok(HeuristicType::NeuralNetwork),
            _ => Err(String::from("Invalid heuristic type identifier!")),
        }
    }
}
