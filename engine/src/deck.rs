use rand::seq::SliceRandom;

use crate::card::{Card, Suit, VALID_RANKS};

#[derive(Clone)]
pub struct Deck {
    cards: Vec<Card>,
}

impl Deck {
    pub fn new() -> Self {
        let cards = Suit::ALL
            .iter()
            .flat_map(|&suit| VALID_RANKS.iter().map(move |&rank| Card::new(suit, rank)))
            .collect();
        Self { cards }
    }

    pub fn shuffle(&mut self) {
        let mut rng = rand::thread_rng();
        self.cards.shuffle(&mut rng);
    }

    pub fn draw(&mut self) -> Option<Card> {
        self.cards.pop()
    }

    pub fn draw_n(&mut self, n: usize) -> Vec<Card> {
        let start = self.cards.len().saturating_sub(n);
        self.cards.split_off(start)
    }

    pub fn remaining(&self) -> usize {
        self.cards.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }

    pub fn from_cards(cards: Vec<Card>) -> Self {
        Self { cards }
    }

    pub fn cards(&self) -> &[Card] {
        &self.cards
    }
}

impl Default for Deck {
    fn default() -> Self {
        Self::new()
    }
}
