use crate::card::Card;

#[derive(Debug, Clone)]
pub struct Player {
    pub name: String,
    pub hand: Vec<Card>,
    pub captured: Vec<Card>,
    pub escobas: u32,
}

impl Player {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            hand: Vec::new(),
            captured: Vec::new(),
            escobas: 0,
        }
    }

    pub fn add_to_hand(&mut self, cards: Vec<Card>) {
        self.hand.extend(cards);
    }

    pub fn remove_from_hand(&mut self, card: &Card) -> Option<Card> {
        let pos = self.hand.iter().position(|c| c == card)?;
        Some(self.hand.remove(pos))
    }

    pub fn capture(&mut self, cards: Vec<Card>) {
        self.captured.extend(cards);
    }

    pub fn record_escoba(&mut self) {
        self.escobas += 1;
    }

    pub fn hand_is_empty(&self) -> bool {
        self.hand.is_empty()
    }
}
