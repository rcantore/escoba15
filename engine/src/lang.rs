use serde::{Serialize, Deserialize};

use crate::card::{Card, Suit};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Lang {
    Es,
    En,
}

impl Suit {
    pub fn localized(&self, lang: Lang) -> &'static str {
        match (self, lang) {
            (Suit::Oros, Lang::Es) => "Oros",
            (Suit::Copas, Lang::Es) => "Copas",
            (Suit::Espadas, Lang::Es) => "Espadas",
            (Suit::Bastos, Lang::Es) => "Bastos",
            (Suit::Oros, Lang::En) => "Coins",
            (Suit::Copas, Lang::En) => "Cups",
            (Suit::Espadas, Lang::En) => "Swords",
            (Suit::Bastos, Lang::En) => "Clubs",
        }
    }
}

impl Card {
    pub fn localized_name(&self, lang: Lang) -> String {
        let suit = self.suit.localized(lang);
        match (self.rank, lang) {
            (1, Lang::Es) => format!("As de {suit}"),
            (1, Lang::En) => format!("Ace of {suit}"),
            (2..=7, Lang::Es) => format!("{} de {suit}", self.rank),
            (2..=7, Lang::En) => format!("{} of {suit}", self.rank),
            (10, Lang::Es) => format!("Sota de {suit}"),
            (10, Lang::En) => format!("Jack of {suit}"),
            (11, Lang::Es) => format!("Caballo de {suit}"),
            (11, Lang::En) => format!("Knight of {suit}"),
            (12, Lang::Es) => format!("Rey de {suit}"),
            (12, Lang::En) => format!("King of {suit}"),
            _ => unreachable!(),
        }
    }
}
