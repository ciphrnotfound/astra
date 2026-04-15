//! Wire messages for CIPHER MVP (JSON over WebSocket).

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    Join { name: String },
    Input {
        dx: i8,
        dy: i8,
        #[serde(default)]
        jump: bool,
    },
    Vote {
        choice: u8,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerView {
    pub id: Uuid,
    pub name: String,
    pub x: f32,
    pub y: f32,
    pub color: String,
    pub hp: f32,
    pub alive: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    Welcome {
        id: Uuid,
        arena_w: f32,
        arena_h: f32,
    },
    State {
        tick: u64,
        players: Vec<PlayerView>,
    },
    VoteOffer {
        ends_at_tick: u64,
        options: [String; 4],
    },
    Error {
        message: String,
    },
}
