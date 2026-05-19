use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub schema_version: String,
    pub game_id: String,
    pub puzzle_id: String,
    pub date: String,
    pub status: String,
    pub guess_count: u32,
    pub max_guesses: u32,
    pub current_stage: u32,
    pub public_state: serde_json::Value,
}
