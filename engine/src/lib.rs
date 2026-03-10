pub mod card;
pub mod deck;
pub mod player;
pub mod game;
pub mod scoring;
pub mod ai;
pub mod lang;

pub use card::{Card, Suit, VALID_RANKS};
pub use deck::Deck;
pub use player::Player;
pub use game::{Game, GameState, PlayResult, GameError, Play, Capture};
pub use scoring::{ScoreBreakdown, calculate_score, is_game_over};
pub use ai::{AiMove, Difficulty, suggest_play};
pub use lang::Lang;

#[cfg(feature = "wasm")]
pub mod wasm;
