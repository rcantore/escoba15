#![cfg(feature = "wasm")]

use wasm_bindgen::prelude::*;
use serde_json::{json, Value};

use crate::ai::{suggest_play, Difficulty};
use crate::game::{Game, GameState, PlayResult};
use crate::lang::Lang;
use crate::scoring::{calculate_score, is_game_over};

/// Serializes a card to a JSON value including computed fields.
fn card_to_json(card: &crate::card::Card, lang: Lang) -> Value {
    json!({
        "suit": format!("{:?}", card.suit),
        "rank": card.rank,
        "value": card.value(),
        "name": card.localized_name(lang),
    })
}

/// Parses a language string into a `Lang` enum, defaulting to Spanish.
fn parse_lang(s: &str) -> Lang {
    match s.to_lowercase().as_str() {
        "en" => Lang::En,
        _ => Lang::Es,
    }
}

/// Parses a difficulty string into a `Difficulty` enum, defaulting to Medium.
fn parse_difficulty(s: &str) -> Difficulty {
    match s.to_lowercase().as_str() {
        "easy" => Difficulty::Easy,
        "hard" => Difficulty::Hard,
        _ => Difficulty::Medium,
    }
}

#[wasm_bindgen]
pub struct WasmGame {
    game: Game,
    lang: Lang,
}

#[wasm_bindgen]
impl WasmGame {
    /// Creates a new game with two player names and a language code ("es" or "en").
    #[wasm_bindgen(constructor)]
    pub fn new(p1_name: &str, p2_name: &str, lang: &str) -> WasmGame {
        WasmGame {
            game: Game::new(p1_name, p2_name),
            lang: parse_lang(lang),
        }
    }

    /// Resets the game for a new round (fresh deck, clear piles) and deals.
    pub fn new_round(&mut self) {
        self.game.new_round();
        self.game.deal_round();
    }

    /// Deals a new round of cards to both players (and to the table on the first deal).
    pub fn deal_round(&mut self) {
        self.game.deal_round();
    }

    /// Returns the full visible game state as a JSON string.
    ///
    /// Shape:
    /// ```json
    /// {
    ///   "state": "Playing",
    ///   "current_player": 0,
    ///   "table": [{ "suit", "rank", "value", "name" }, ...],
    ///   "players": [
    ///     { "name", "hand_size", "captured_count", "escobas" },
    ///     ...
    ///   ],
    ///   "deck_remaining": 30
    /// }
    /// ```
    pub fn get_state(&self) -> String {
        let table: Vec<Value> = self.game.table
            .iter()
            .map(|c| card_to_json(c, self.lang))
            .collect();

        let players: Vec<Value> = self.game.players
            .iter()
            .map(|p| {
                json!({
                    "name": p.name,
                    "hand_size": p.hand.len(),
                    "captured_count": p.captured.len(),
                    "escobas": p.escobas,
                })
            })
            .collect();

        let state_str = match self.game.state {
            GameState::Dealing => "Dealing",
            GameState::Playing => "Playing",
            GameState::RoundEnd => "RoundEnd",
            GameState::GameOver => "GameOver",
        };

        let result = json!({
            "state": state_str,
            "current_player": self.game.current_player,
            "table": table,
            "players": players,
            "deck_remaining": self.game.deck.remaining(),
        });

        serde_json::to_string(&result).unwrap_or_else(|e| {
            json!({"error": e.to_string()}).to_string()
        })
    }

    /// Returns the hand cards for the given player as a JSON array.
    ///
    /// Shape: `[{ "suit", "rank", "value", "name" }, ...]`
    pub fn get_hand(&self, player_index: usize) -> String {
        if player_index >= 2 {
            return json!({"error": "Invalid player index"}).to_string();
        }

        let hand: Vec<Value> = self.game.players[player_index]
            .hand
            .iter()
            .map(|c| card_to_json(c, self.lang))
            .collect();

        serde_json::to_string(&hand).unwrap_or_else(|e| {
            json!({"error": e.to_string()}).to_string()
        })
    }

    /// Returns all valid plays for the current player as a JSON string.
    ///
    /// Shape:
    /// ```json
    /// [
    ///   {
    ///     "hand_index": 0,
    ///     "hand_card": { "suit", "rank", "value", "name" },
    ///     "captures": [
    ///       { "table_indices": [0, 2], "table_cards": [{ ... }, { ... }] }
    ///     ],
    ///     "can_drop": true
    ///   },
    ///   ...
    /// ]
    /// ```
    pub fn valid_plays(&self) -> String {
        let plays = self.game.valid_plays();
        let lang = self.lang;

        let result: Vec<Value> = plays
            .iter()
            .map(|play| {
                let captures: Vec<Value> = play.captures
                    .iter()
                    .map(|cap| {
                        let table_cards: Vec<Value> = cap.table_cards
                            .iter()
                            .map(|c| card_to_json(c, lang))
                            .collect();
                        json!({
                            "table_indices": cap.table_indices,
                            "table_cards": table_cards,
                        })
                    })
                    .collect();

                json!({
                    "hand_index": play.hand_index,
                    "hand_card": card_to_json(&play.hand_card, lang),
                    "captures": captures,
                    "can_drop": play.can_drop,
                })
            })
            .collect();

        serde_json::to_string(&result).unwrap_or_else(|e| {
            json!({"error": e.to_string()}).to_string()
        })
    }

    /// Plays a card from the current player's hand.
    ///
    /// - `hand_card_idx`: index into the current player's hand.
    /// - `table_indices_json`: JSON string — `"null"` to drop, or `"[0, 2]"` to capture.
    ///
    /// Automatically advances the turn (and deals a new round if needed) on success.
    ///
    /// Returns JSON:
    /// ```json
    /// { "result": "captured", "cards": [...], "escoba": false }
    /// { "result": "dropped" }
    /// { "error": "InvalidCombination" }
    /// ```
    pub fn play_card(&mut self, hand_card_idx: usize, table_indices_json: &str) -> String {
        let table_indices: Option<Vec<usize>> = match table_indices_json.trim() {
            "null" | "" => None,
            s => match serde_json::from_str(s) {
                Ok(indices) => Some(indices),
                Err(e) => {
                    return json!({"error": format!("Invalid table_indices JSON: {e}")}).to_string();
                }
            },
        };

        match self.game.play_card(hand_card_idx, table_indices) {
            Ok(play_result) => {
                // Advance turn (and deal if needed) automatically
                self.game.next_turn();

                match play_result {
                    PlayResult::Captured { cards, escoba } => {
                        let cards_json: Vec<Value> = cards
                            .iter()
                            .map(|c| card_to_json(c, self.lang))
                            .collect();
                        json!({
                            "result": "captured",
                            "cards": cards_json,
                            "escoba": escoba,
                        })
                        .to_string()
                    }
                    PlayResult::Dropped => {
                        json!({"result": "dropped"}).to_string()
                    }
                }
            }
            Err(e) => {
                let error_str = match e {
                    crate::game::GameError::InvalidCard => "InvalidCard",
                    crate::game::GameError::InvalidCombination => "InvalidCombination",
                    crate::game::GameError::NotYourTurn => "NotYourTurn",
                    crate::game::GameError::GameNotInPlay => "GameNotInPlay",
                };
                json!({"error": error_str}).to_string()
            }
        }
    }

    /// Suggests an AI move for the current player.
    ///
    /// - `difficulty`: `"easy"`, `"medium"`, or `"hard"`.
    ///
    /// Returns JSON: `{ "hand_index": 0, "table_indices": [1, 3] }` or
    /// `{ "hand_index": 0, "table_indices": null }` for a drop.
    pub fn ai_suggest(&self, difficulty: &str) -> String {
        let diff = parse_difficulty(difficulty);
        let ai_move = suggest_play(&self.game, diff);

        json!({
            "hand_index": ai_move.hand_index,
            "table_indices": ai_move.table_indices,
        })
        .to_string()
    }

    /// Calculates scores for both players based on their captured cards.
    ///
    /// Returns JSON:
    /// ```json
    /// [
    ///   { "name": "Alice", "breakdown": { "cards_point", "oros_point", ... "total" } },
    ///   { "name": "Bob",   "breakdown": { ... } }
    /// ]
    /// ```
    pub fn calculate_scores(&self) -> String {
        let p0 = &self.game.players[0];
        let p1 = &self.game.players[1];

        let score0 = calculate_score(&p0.captured, p0.escobas, &p1.captured);
        let score1 = calculate_score(&p1.captured, p1.escobas, &p0.captured);

        let result = json!([
            {
                "name": p0.name,
                "breakdown": {
                    "cards_point": score0.cards_point,
                    "oros_point": score0.oros_point,
                    "siete_velo_point": score0.siete_velo_point,
                    "sevens_point": score0.sevens_point,
                    "escobas_points": score0.escobas_points,
                    "total": score0.total,
                },
            },
            {
                "name": p1.name,
                "breakdown": {
                    "cards_point": score1.cards_point,
                    "oros_point": score1.oros_point,
                    "siete_velo_point": score1.siete_velo_point,
                    "sevens_point": score1.sevens_point,
                    "escobas_points": score1.escobas_points,
                    "total": score1.total,
                },
            },
        ]);

        serde_json::to_string(&result).unwrap_or_else(|e| {
            json!({"error": e.to_string()}).to_string()
        })
    }

    /// Checks whether the game is over given accumulated scores.
    ///
    /// - `scores_json`: JSON string in the form `[[name, score], [name, score]]`,
    ///   e.g. `[["Alice", 16], ["Bob", 12]]`.
    ///
    /// Returns JSON: `{ "over": true, "winner": "Alice" }` or `{ "over": false }`.
    pub fn is_game_over_check(&self, scores_json: &str) -> String {
        let parsed: Result<Vec<(String, u32)>, _> = serde_json::from_str(scores_json);

        match parsed {
            Ok(scores) => match is_game_over(&scores) {
                Some(winner) => json!({"over": true, "winner": winner}).to_string(),
                None => json!({"over": false}).to_string(),
            },
            Err(e) => {
                json!({"error": format!("Invalid scores JSON: {e}")}).to_string()
            }
        }
    }

    /// Sets the display language for card names.
    ///
    /// - `lang`: `"es"` for Spanish, `"en"` for English.
    pub fn set_lang(&mut self, lang: &str) {
        self.lang = parse_lang(lang);
    }
}
