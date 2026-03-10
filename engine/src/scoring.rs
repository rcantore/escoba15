use serde::{Serialize, Deserialize};

use crate::card::{Card, Suit};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScoreBreakdown {
    pub cards_point: u32,
    pub oros_point: u32,
    pub siete_velo_point: u32,
    pub sevens_point: u32,
    pub escobas_points: u32,
    pub total: u32,
}

pub fn calculate_score(
    captured: &[Card],
    escobas: u32,
    opponent_captured: &[Card],
) -> ScoreBreakdown {
    let cards_point = compare_count(captured.len(), opponent_captured.len());
    let oros_point = compare_count(count_oros(captured), count_oros(opponent_captured));
    let siete_velo_point = if has_siete_de_velo(captured) { 1 } else { 0 };
    let sevens_point = compare_count(count_sevens(captured), count_sevens(opponent_captured));
    let escobas_points = escobas;

    let total = cards_point + oros_point + siete_velo_point + sevens_point + escobas_points;

    ScoreBreakdown {
        cards_point,
        oros_point,
        siete_velo_point,
        sevens_point,
        escobas_points,
        total,
    }
}

pub fn is_game_over(scores: &[(String, u32)]) -> Option<String> {
    let mut qualifiers: Vec<&(String, u32)> = scores
        .iter()
        .filter(|(_, score)| *score >= 15)
        .collect();

    if qualifiers.is_empty() {
        return None;
    }

    qualifiers.sort_by(|a, b| b.1.cmp(&a.1));

    if qualifiers.len() >= 2 && qualifiers[0].1 == qualifiers[1].1 {
        return None;
    }

    Some(qualifiers[0].0.clone())
}

fn compare_count(player: usize, opponent: usize) -> u32 {
    if player > opponent { 1 } else { 0 }
}

fn count_oros(cards: &[Card]) -> usize {
    cards.iter().filter(|c| c.suit == Suit::Oros).count()
}

fn count_sevens(cards: &[Card]) -> usize {
    cards.iter().filter(|c| c.rank == 7).count()
}

fn has_siete_de_velo(cards: &[Card]) -> bool {
    cards.iter().any(|c| c.suit == Suit::Oros && c.rank == 7)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn card(suit: Suit, rank: u8) -> Card {
        Card::new(suit, rank)
    }

    #[test]
    fn test_most_cards_wins_point() {
        let captured = vec![card(Suit::Oros, 1), card(Suit::Copas, 2), card(Suit::Bastos, 3)];
        let opponent = vec![card(Suit::Espadas, 4), card(Suit::Copas, 5)];
        let score = calculate_score(&captured, 0, &opponent);
        assert_eq!(score.cards_point, 1);
    }

    #[test]
    fn test_tied_cards_no_point() {
        let captured = vec![card(Suit::Oros, 1), card(Suit::Copas, 2)];
        let opponent = vec![card(Suit::Espadas, 3), card(Suit::Bastos, 4)];
        let score = calculate_score(&captured, 0, &opponent);
        assert_eq!(score.cards_point, 0);
    }

    #[test]
    fn test_most_oros_wins_point() {
        let captured = vec![card(Suit::Oros, 1), card(Suit::Oros, 3)];
        let opponent = vec![card(Suit::Oros, 5), card(Suit::Copas, 6)];
        let score = calculate_score(&captured, 0, &opponent);
        assert_eq!(score.oros_point, 1);
    }

    #[test]
    fn test_siete_de_velo() {
        let captured = vec![card(Suit::Oros, 7)];
        let opponent = vec![card(Suit::Copas, 7)];
        let score = calculate_score(&captured, 0, &opponent);
        assert_eq!(score.siete_velo_point, 1);
    }

    #[test]
    fn test_no_siete_de_velo() {
        let captured = vec![card(Suit::Copas, 7)];
        let opponent = vec![card(Suit::Oros, 7)];
        let score = calculate_score(&captured, 0, &opponent);
        assert_eq!(score.siete_velo_point, 0);
    }

    #[test]
    fn test_most_sevens_wins_point() {
        let captured = vec![card(Suit::Oros, 7), card(Suit::Copas, 7)];
        let opponent = vec![card(Suit::Espadas, 7)];
        let score = calculate_score(&captured, 0, &opponent);
        assert_eq!(score.sevens_point, 1);
    }

    #[test]
    fn test_escobas_add_points() {
        let captured = vec![card(Suit::Copas, 1)];
        let opponent = vec![card(Suit::Bastos, 2)];
        let score = calculate_score(&captured, 3, &opponent);
        assert_eq!(score.escobas_points, 3);
    }

    #[test]
    fn test_total_sums_all() {
        // Player has more cards (1), more oros (1), siete de velo (1), more sevens (1), 2 escobas
        let captured = vec![
            card(Suit::Oros, 7),
            card(Suit::Oros, 1),
            card(Suit::Copas, 7),
            card(Suit::Bastos, 3),
            card(Suit::Espadas, 5),
        ];
        let opponent = vec![card(Suit::Copas, 1), card(Suit::Bastos, 2)];
        let score = calculate_score(&captured, 2, &opponent);
        assert_eq!(score.total, 6); // 1 + 1 + 1 + 1 + 2
    }

    #[test]
    fn test_game_not_over_below_15() {
        let scores = vec![
            ("Alice".to_string(), 10),
            ("Bob".to_string(), 12),
        ];
        assert_eq!(is_game_over(&scores), None);
    }

    #[test]
    fn test_game_over_single_winner() {
        let scores = vec![
            ("Alice".to_string(), 16),
            ("Bob".to_string(), 12),
        ];
        assert_eq!(is_game_over(&scores), Some("Alice".to_string()));
    }

    #[test]
    fn test_game_over_both_above_15_higher_wins() {
        let scores = vec![
            ("Alice".to_string(), 15),
            ("Bob".to_string(), 17),
        ];
        assert_eq!(is_game_over(&scores), Some("Bob".to_string()));
    }

    #[test]
    fn test_game_over_tied_at_15_returns_none() {
        let scores = vec![
            ("Alice".to_string(), 15),
            ("Bob".to_string(), 15),
        ];
        assert_eq!(is_game_over(&scores), None);
    }
}
