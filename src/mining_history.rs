use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiningPattern {
    pub challenge: Vec<u8>,
    pub nonce_range: (u64, u64),
    pub difficulty: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MiningHistory {
    patterns: Vec<MiningPattern>,
}

impl MiningHistory {
    pub fn new() -> Self {
        if let Ok(content) = fs::read_to_string("mining_history.json") {
            if let Ok(history) = serde_json::from_str(&content) {
                return history;
            }
        }
        Self {
            patterns: Vec::new(),
        }
    }

    pub fn add_pattern(&mut self, pattern: MiningPattern) {
        self.patterns.push(pattern);
        self.save();
    }

    pub fn get_best_patterns(&self, target_difficulty: u32) -> Vec<&MiningPattern> {
        self.patterns
            .iter()
            .filter(|p| p.difficulty >= target_difficulty)
            .collect()
    }

    fn save(&self) {
        if let Ok(content) = serde_json::to_string_pretty(self) {
            let _ = fs::write("mining_history.json", content);
        }
    }
}
