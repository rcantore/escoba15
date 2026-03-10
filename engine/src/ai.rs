use serde::{Serialize, Deserialize};
use rand::seq::SliceRandom;
use rand::thread_rng;

use crate::card::Card;
use crate::deck::Deck;
use crate::game::{Game, GameState};
use crate::scoring::calculate_score;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AiMove {
    pub hand_index: usize,
    pub table_indices: Option<Vec<usize>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
}

impl Difficulty {
    pub fn simulations(self) -> u32 {
        match self {
            Difficulty::Easy => 100,
            Difficulty::Medium => 1000,
            Difficulty::Hard => 10000,
        }
    }
}

/// Suggests the best move for the current player using MCTS.
///
/// Uses Information Set Monte Carlo Tree Search (ISMCTS):
/// the AI doesn't know the opponent's hand or deck order,
/// so each simulation randomizes the hidden cards.
pub fn suggest_play(game: &Game, difficulty: Difficulty) -> AiMove {
    let ai_player = game.current_player;
    let moves = enumerate_moves(game);

    if moves.len() == 1 {
        return moves.into_iter().next().unwrap();
    }

    let simulations = difficulty.simulations();
    let sims_per_move = simulations / moves.len() as u32;

    let mut best_move = moves[0].clone();
    let mut best_score = f64::NEG_INFINITY;

    for ai_move in &moves {
        let mut wins = 0.0;
        let total = sims_per_move.max(1);

        for _ in 0..total {
            let mut sim = determinize(game, ai_player);

            // Apply our chosen move
            let _ = sim.play_card(
                ai_move.hand_index,
                ai_move.table_indices.clone(),
            );
            sim.next_turn();

            // Play out the rest of the round randomly
            playout_round(&mut sim);

            // Score the result from our perspective
            let opponent = 1 - ai_player;
            let our_score = calculate_score(
                &sim.players[ai_player].captured,
                sim.players[ai_player].escobas,
                &sim.players[opponent].captured,
            );
            let their_score = calculate_score(
                &sim.players[opponent].captured,
                sim.players[opponent].escobas,
                &sim.players[ai_player].captured,
            );

            // Win = +1, draw = +0.5, loss = 0
            if our_score.total > their_score.total {
                wins += 1.0;
            } else if our_score.total == their_score.total {
                wins += 0.5;
            }
        }

        let win_rate = wins / total as f64;
        if win_rate > best_score {
            best_score = win_rate;
            best_move = ai_move.clone();
        }
    }

    best_move
}

/// Enumerates all legal moves for the current player.
fn enumerate_moves(game: &Game) -> Vec<AiMove> {
    let mut moves = Vec::new();

    for play in game.valid_plays() {
        if play.captures.is_empty() {
            // Can only drop
            moves.push(AiMove {
                hand_index: play.hand_index,
                table_indices: None,
            });
        } else {
            // Each capture option is a separate move
            for capture in &play.captures {
                moves.push(AiMove {
                    hand_index: play.hand_index,
                    table_indices: Some(capture.table_indices.clone()),
                });
            }
            // Can also choose to drop instead of capturing
            moves.push(AiMove {
                hand_index: play.hand_index,
                table_indices: None,
            });
        }
    }

    moves
}

/// Creates a determinized copy of the game state.
///
/// The AI knows its own hand and the table, but not the opponent's
/// hand or the remaining deck order. This function collects all
/// unknown cards, shuffles them, and redistributes them to the
/// opponent's hand and the deck.
fn determinize(game: &Game, ai_player: usize) -> Game {
    let mut sim = game.clone();
    let opponent = 1 - ai_player;
    let mut rng = thread_rng();

    // Collect all cards the AI can't see
    let mut hidden: Vec<Card> = Vec::new();
    hidden.extend_from_slice(&sim.players[opponent].hand);
    hidden.extend_from_slice(sim.deck.cards());

    hidden.shuffle(&mut rng);

    // Redistribute: opponent gets back the same number of cards
    let opp_hand_size = sim.players[opponent].hand.len();
    sim.players[opponent].hand = hidden.drain(..opp_hand_size).collect();

    // Remaining hidden cards become the new deck
    sim.deck = Deck::from_cards(hidden);

    sim
}

/// Plays out the rest of a round with random moves.
fn playout_round(game: &mut Game) {
    let mut rng = thread_rng();

    while game.state == GameState::Playing {
        let moves = enumerate_moves(game);
        if moves.is_empty() {
            break;
        }
        let chosen = moves.choose(&mut rng).unwrap();
        let _ = game.play_card(chosen.hand_index, chosen.table_indices.clone());
        game.next_turn();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::card::{Card, Suit};
    use crate::game::GameState;

    fn card(suit: Suit, rank: u8) -> Card {
        Card::new(suit, rank)
    }

    #[test]
    fn suggest_play_returns_valid_move() {
        let mut game = Game::new("Human", "AI");
        game.deal_round();
        game.current_player = 1; // AI's turn
        let ai_move = suggest_play(&game, Difficulty::Easy);
        assert!(ai_move.hand_index < game.players[1].hand.len());
    }

    #[test]
    fn suggest_play_prefers_capture_over_drop() {
        let mut game = Game::new("Human", "AI");
        game.state = GameState::Playing;
        game.current_player = 1;
        // AI has a 7, table has Sota (val 8) => 7+8=15, guaranteed capture
        game.players[1].hand = vec![card(Suit::Oros, 7)];
        game.table = vec![card(Suit::Copas, 10)]; // val 8

        let ai_move = suggest_play(&game, Difficulty::Medium);
        // With enough simulations, AI should prefer capturing
        assert!(
            ai_move.table_indices.is_some(),
            "AI should capture when 7+8=15, got drop instead"
        );
    }

    #[test]
    fn enumerate_moves_includes_drop_and_captures() {
        let mut game = Game::new("A", "B");
        game.state = GameState::Playing;
        game.current_player = 0;
        game.players[0].hand = vec![card(Suit::Oros, 5)];
        game.table = vec![
            card(Suit::Copas, 12), // val 10 => 5+10=15
            card(Suit::Espadas, 3),
            card(Suit::Bastos, 7), // 3+7=10 => 5+3+7=15
        ];

        let moves = enumerate_moves(&game);
        // Should have: capture [0], capture [1,2], drop
        assert_eq!(moves.len(), 3);
    }

    #[test]
    fn determinize_preserves_ai_hand() {
        let mut game = Game::new("Human", "AI");
        game.deal_round();
        let ai_hand = game.players[1].hand.clone();
        let sim = determinize(&game, 1);
        assert_eq!(sim.players[1].hand, ai_hand);
    }

    #[test]
    fn determinize_preserves_table() {
        let mut game = Game::new("Human", "AI");
        game.deal_round();
        let table = game.table.clone();
        let sim = determinize(&game, 1);
        assert_eq!(sim.table, table);
    }

    #[test]
    fn determinize_preserves_total_card_count() {
        let mut game = Game::new("Human", "AI");
        game.deal_round();
        let total_before = game.players[0].hand.len()
            + game.players[1].hand.len()
            + game.table.len()
            + game.deck.remaining()
            + game.players[0].captured.len()
            + game.players[1].captured.len();
        let sim = determinize(&game, 1);
        let total_after = sim.players[0].hand.len()
            + sim.players[1].hand.len()
            + sim.table.len()
            + sim.deck.remaining()
            + sim.players[0].captured.len()
            + sim.players[1].captured.len();
        assert_eq!(total_before, total_after);
    }

    #[test]
    fn playout_round_reaches_round_end() {
        let mut game = Game::new("A", "B");
        game.deal_round();
        playout_round(&mut game);
        assert_eq!(game.state, GameState::RoundEnd);
    }

    #[test]
    fn difficulty_simulations_scale() {
        assert!(Difficulty::Easy.simulations() < Difficulty::Medium.simulations());
        assert!(Difficulty::Medium.simulations() < Difficulty::Hard.simulations());
    }
}
