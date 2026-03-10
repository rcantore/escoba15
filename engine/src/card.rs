use serde::{Serialize, Deserialize};
use std::fmt;

pub const VALID_RANKS: [u8; 10] = [1, 2, 3, 4, 5, 6, 7, 10, 11, 12];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Suit {
    Oros,
    Copas,
    Espadas,
    Bastos,
}

impl Suit {
    pub const ALL: [Suit; 4] = [Suit::Oros, Suit::Copas, Suit::Espadas, Suit::Bastos];
}

impl fmt::Display for Suit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Suit::Oros => write!(f, "Oros"),
            Suit::Copas => write!(f, "Copas"),
            Suit::Espadas => write!(f, "Espadas"),
            Suit::Bastos => write!(f, "Bastos"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Card {
    pub suit: Suit,
    pub rank: u8,
}

impl Card {
    pub fn new(suit: Suit, rank: u8) -> Self {
        assert!(
            VALID_RANKS.contains(&rank),
            "Invalid rank: {rank}. Valid ranks are 1-7, 10-12"
        );
        Self { suit, rank }
    }

    pub fn value(&self) -> u8 {
        match self.rank {
            1..=7 => self.rank,
            10 => 8,
            11 => 9,
            12 => 10,
            _ => unreachable!(),
        }
    }

    pub fn display_name(&self) -> String {
        let rank_name = match self.rank {
            1 => "As",
            2..=7 => return format!("{} de {}", self.rank, self.suit),
            10 => "Sota",
            11 => "Caballo",
            12 => "Rey",
            _ => unreachable!(),
        };
        format!("{rank_name} de {}", self.suit)
    }
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}
