use serde::{Serialize, Deserialize};

use crate::card::Card;
use crate::deck::Deck;
use crate::player::Player;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameState {
    Dealing,
    Playing,
    RoundEnd,
    GameOver,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlayResult {
    Captured { cards: Vec<Card>, escoba: bool },
    Dropped,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameError {
    InvalidCard,
    InvalidCombination,
    NotYourTurn,
    GameNotInPlay,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Play {
    pub hand_index: usize,
    pub hand_card: Card,
    pub captures: Vec<Capture>,
    pub can_drop: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Capture {
    pub table_indices: Vec<usize>,
    pub table_cards: Vec<Card>,
}

#[derive(Clone)]
pub struct Game {
    pub deck: Deck,
    pub players: [Player; 2],
    pub table: Vec<Card>,
    pub current_player: usize,
    pub state: GameState,
    pub last_capturer: Option<usize>,
    first_deal: bool,
}

impl Game {
    pub fn new(p1_name: &str, p2_name: &str) -> Self {
        let mut deck = Deck::new();
        deck.shuffle();
        Self {
            deck,
            players: [Player::new(p1_name), Player::new(p2_name)],
            table: Vec::new(),
            current_player: 0,
            state: GameState::Dealing,
            last_capturer: None,
            first_deal: true,
        }
    }

    /// Resets the game for a new round: fresh shuffled deck, clear hands,
    /// clear captured piles, clear table. Call `deal_round()` after this.
    pub fn new_round(&mut self) {
        self.deck = Deck::new();
        self.deck.shuffle();
        self.table.clear();
        for p in &mut self.players {
            p.hand.clear();
            p.captured.clear();
            p.escobas = 0;
        }
        self.first_deal = true;
        self.last_capturer = None;
        self.state = GameState::Dealing;
    }

    /// Deals 3 cards to each player. On the first deal of the round,
    /// also deals 4 cards to the table.
    pub fn deal_round(&mut self) {
        for i in 0..2 {
            let cards = self.deck.draw_n(3);
            self.players[i].add_to_hand(cards);
        }

        if self.first_deal {
            let table_cards = self.deck.draw_n(4);
            self.table.extend(table_cards);
            self.first_deal = false;
        }

        self.state = GameState::Playing;
    }

    /// Finds all subsets of table card indices whose values, combined with
    /// the hand card's value, sum to 15.
    ///
    /// Uses bit manipulation to enumerate all 2^n subsets of the table.
    pub fn find_combinations(hand_card: &Card, table: &[Card]) -> Vec<Vec<usize>> {
        let hand_value = hand_card.value() as u32;
        let n = table.len();
        let mut results = Vec::new();

        // Enumerate all non-empty subsets via bitmask
        for mask in 1..(1u32 << n) {
            let mut subset_sum: u32 = 0;
            let mut indices = Vec::new();

            for bit in 0..n {
                if mask & (1 << bit) != 0 {
                    subset_sum += table[bit].value() as u32;
                    indices.push(bit);
                }
            }

            if hand_value + subset_sum == 15 {
                results.push(indices);
            }
        }

        results
    }

    /// Plays a card from the current player's hand.
    ///
    /// - If `table_card_indices` is `Some`, validates that those table cards
    ///   plus the hand card sum to 15 and captures them.
    /// - If `None`, the hand card is dropped onto the table.
    /// - Detects an escoba when the table is emptied by a capture.
    pub fn play_card(
        &mut self,
        hand_card_idx: usize,
        table_card_indices: Option<Vec<usize>>,
    ) -> Result<PlayResult, GameError> {
        if self.state != GameState::Playing {
            return Err(GameError::GameNotInPlay);
        }

        let player = &self.players[self.current_player];
        if hand_card_idx >= player.hand.len() {
            return Err(GameError::InvalidCard);
        }

        let hand_card = player.hand[hand_card_idx];

        match table_card_indices {
            Some(indices) => {
                // Validate indices are in range and unique
                for &idx in &indices {
                    if idx >= self.table.len() {
                        return Err(GameError::InvalidCombination);
                    }
                }
                let mut sorted = indices.clone();
                sorted.sort_unstable();
                sorted.dedup();
                if sorted.len() != indices.len() {
                    return Err(GameError::InvalidCombination);
                }

                // Validate that values sum to 15
                let table_sum: u32 = indices
                    .iter()
                    .map(|&i| self.table[i].value() as u32)
                    .sum();
                if hand_card.value() as u32 + table_sum != 15 {
                    return Err(GameError::InvalidCombination);
                }

                // Remove hand card
                self.players[self.current_player].hand.remove(hand_card_idx);

                // Collect captured table cards (remove from highest index first
                // to preserve lower indices)
                let mut capture_indices = indices.clone();
                capture_indices.sort_unstable();
                let mut captured_cards = Vec::with_capacity(capture_indices.len() + 1);
                for &idx in capture_indices.iter().rev() {
                    captured_cards.push(self.table.remove(idx));
                }
                captured_cards.push(hand_card);

                // Detect escoba
                let escoba = self.table.is_empty();
                if escoba {
                    self.players[self.current_player].record_escoba();
                }

                let result_cards = captured_cards.clone();
                self.players[self.current_player].capture(captured_cards);
                self.last_capturer = Some(self.current_player);

                Ok(PlayResult::Captured {
                    cards: result_cards,
                    escoba,
                })
            }
            None => {
                // Drop the card onto the table
                self.players[self.current_player].hand.remove(hand_card_idx);
                self.table.push(hand_card);
                Ok(PlayResult::Dropped)
            }
        }
    }

    /// Returns all valid plays for the current player.
    /// Each `Play` represents one hand card and all its possible captures.
    pub fn valid_plays(&self) -> Vec<Play> {
        let player = &self.players[self.current_player];
        player
            .hand
            .iter()
            .enumerate()
            .map(|(hand_index, hand_card)| {
                let combos = Self::find_combinations(hand_card, &self.table);
                let captures = combos
                    .into_iter()
                    .map(|indices| {
                        let table_cards = indices.iter().map(|&i| self.table[i]).collect();
                        Capture {
                            table_indices: indices,
                            table_cards,
                        }
                    })
                    .collect::<Vec<_>>();
                let can_drop = true;
                Play {
                    hand_index,
                    hand_card: *hand_card,
                    captures,
                    can_drop,
                }
            })
            .collect()
    }

    /// Advances to the next turn. Switches the current player.
    /// If both players' hands are empty, deals a new round or
    /// transitions to `RoundEnd` when the deck is exhausted.
    pub fn next_turn(&mut self) {
        self.current_player = 1 - self.current_player;

        if self.players[0].hand_is_empty() && self.players[1].hand_is_empty() {
            if self.deck.is_empty() {
                // Round is over; remaining table cards go to last capturer
                if !self.table.is_empty() {
                    if let Some(capturer) = self.last_capturer {
                        let remaining: Vec<Card> = self.table.drain(..).collect();
                        self.players[capturer].capture(remaining);
                    }
                }
                self.state = GameState::RoundEnd;
            } else {
                self.deal_round();
            }
        }
    }
}
