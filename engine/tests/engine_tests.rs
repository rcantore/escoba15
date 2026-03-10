use escoba15_engine::card::{Card, Suit, VALID_RANKS};
use escoba15_engine::deck::Deck;
use escoba15_engine::game::{Game, GameError, GameState, PlayResult};
use escoba15_engine::player::Player;
use escoba15_engine::scoring::{calculate_score, is_game_over};
use std::collections::HashSet;

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

fn card(suit: Suit, rank: u8) -> Card {
    Card::new(suit, rank)
}

// ===========================================================================
// Card tests
// ===========================================================================

#[test]
fn card_creation_all_valid_ranks() {
    for &rank in &VALID_RANKS {
        let c = Card::new(Suit::Oros, rank);
        assert_eq!(c.rank, rank);
        assert_eq!(c.suit, Suit::Oros);
    }
}

#[test]
fn card_values_1_through_7_map_to_themselves() {
    for rank in 1..=7u8 {
        let c = Card::new(Suit::Copas, rank);
        assert_eq!(c.value(), rank, "rank {} should map to value {}", rank, rank);
    }
}

#[test]
fn card_value_10_maps_to_8() {
    assert_eq!(Card::new(Suit::Espadas, 10).value(), 8);
}

#[test]
fn card_value_11_maps_to_9() {
    assert_eq!(Card::new(Suit::Bastos, 11).value(), 9);
}

#[test]
fn card_value_12_maps_to_10() {
    assert_eq!(Card::new(Suit::Oros, 12).value(), 10);
}

#[test]
#[should_panic(expected = "Invalid rank")]
fn card_invalid_rank_0_panics() {
    Card::new(Suit::Oros, 0);
}

#[test]
#[should_panic(expected = "Invalid rank")]
fn card_invalid_rank_8_panics() {
    Card::new(Suit::Copas, 8);
}

#[test]
#[should_panic(expected = "Invalid rank")]
fn card_invalid_rank_9_panics() {
    Card::new(Suit::Espadas, 9);
}

#[test]
#[should_panic(expected = "Invalid rank")]
fn card_invalid_rank_13_panics() {
    Card::new(Suit::Bastos, 13);
}

// ===========================================================================
// Deck tests
// ===========================================================================

#[test]
fn deck_new_has_40_cards() {
    let deck = Deck::new();
    assert_eq!(deck.remaining(), 40);
}

#[test]
fn deck_all_40_cards_are_unique() {
    let mut deck = Deck::new();
    let mut seen = HashSet::new();
    while let Some(card) = deck.draw() {
        assert!(
            seen.insert((card.suit, card.rank)),
            "duplicate card: {:?}",
            card
        );
    }
    assert_eq!(seen.len(), 40);
}

#[test]
fn deck_draw_reduces_count() {
    let mut deck = Deck::new();
    assert_eq!(deck.remaining(), 40);
    let _ = deck.draw();
    assert_eq!(deck.remaining(), 39);
    let _ = deck.draw();
    assert_eq!(deck.remaining(), 38);
}

#[test]
fn deck_draw_n_returns_correct_number() {
    let mut deck = Deck::new();
    let drawn = deck.draw_n(5);
    assert_eq!(drawn.len(), 5);
    assert_eq!(deck.remaining(), 35);
}

#[test]
fn deck_draw_n_more_than_remaining_returns_remaining() {
    let mut deck = Deck::new();
    let _ = deck.draw_n(38);
    assert_eq!(deck.remaining(), 2);
    let drawn = deck.draw_n(5);
    assert_eq!(drawn.len(), 2);
    assert_eq!(deck.remaining(), 0);
}

#[test]
fn deck_empty_returns_none_on_draw() {
    let mut deck = Deck::new();
    let _ = deck.draw_n(40);
    assert!(deck.draw().is_none());
}

#[test]
fn deck_shuffle_preserves_card_count() {
    let mut deck = Deck::new();
    deck.shuffle();
    assert_eq!(deck.remaining(), 40);
}

// ===========================================================================
// Combination finding tests (heart of the game)
// ===========================================================================

#[test]
fn combinations_hand_7_plus_table_sota_equals_15() {
    // hand card value 7, table card Sota (rank 10, value 8) => 7 + 8 = 15
    let hand = card(Suit::Copas, 7);
    let table = vec![card(Suit::Oros, 10)];
    let combos = Game::find_combinations(&hand, &table);
    assert_eq!(combos.len(), 1);
    assert_eq!(combos[0], vec![0]);
}

#[test]
fn combinations_hand_5_plus_table_3_and_7_equals_15() {
    // hand value 5, table has 3 (val 3) and 7 (val 7) => 5 + 3 + 7 = 15
    let hand = card(Suit::Bastos, 5);
    let table = vec![card(Suit::Copas, 3), card(Suit::Espadas, 7)];
    let combos = Game::find_combinations(&hand, &table);
    assert!(
        combos.contains(&vec![0, 1]),
        "expected [0,1] in {:?}",
        combos
    );
}

#[test]
fn combinations_three_table_cards_sum_to_15() {
    // hand value 1 (As), table has 4, 4, 6 => 1+4+4+6 = 15
    let hand = card(Suit::Oros, 1);
    let table = vec![
        card(Suit::Copas, 4),
        card(Suit::Espadas, 4),
        card(Suit::Bastos, 6),
    ];
    let combos = Game::find_combinations(&hand, &table);
    assert!(
        combos.contains(&vec![0, 1, 2]),
        "expected [0,1,2] in {:?}",
        combos
    );
}

#[test]
fn combinations_no_valid_combination_returns_empty() {
    // hand value 1, table has only a 2 => 1+2=3 != 15
    let hand = card(Suit::Oros, 1);
    let table = vec![card(Suit::Copas, 2)];
    let combos = Game::find_combinations(&hand, &table);
    assert!(combos.is_empty());
}

#[test]
fn combinations_multiple_valid_found() {
    // hand value 5, table has: 10(val 10), 3+7(val 3+7=10), so two combos summing to 10
    // Actually: hand 5, table [Rey(12,val10), 3, 7]
    // combo A: 5 + 10 = 15 => [0]
    // combo B: 5 + 3 + 7 = 15 => [1, 2]
    let hand = card(Suit::Oros, 5);
    let table = vec![
        card(Suit::Copas, 12),  // value 10
        card(Suit::Espadas, 3), // value 3
        card(Suit::Bastos, 7),  // value 7
    ];
    let combos = Game::find_combinations(&hand, &table);
    assert_eq!(combos.len(), 2, "expected 2 combinations, got {:?}", combos);
    assert!(combos.contains(&vec![0]));
    assert!(combos.contains(&vec![1, 2]));
}

#[test]
fn combinations_hand_5_plus_sota_no_match() {
    // hand value 5, Sota (rank 10, value 8) => 5+8=13 != 15
    let hand = card(Suit::Oros, 5);
    let table = vec![card(Suit::Copas, 10)];
    let combos = Game::find_combinations(&hand, &table);
    assert!(combos.is_empty());
}

#[test]
fn combinations_empty_table_returns_empty() {
    let hand = card(Suit::Oros, 5);
    let table: Vec<Card> = vec![];
    let combos = Game::find_combinations(&hand, &table);
    assert!(combos.is_empty());
}

#[test]
fn combinations_hand_card_alone_worth_15_not_returned() {
    // A card worth 10 (Rey) cannot capture an empty set to make 15.
    // find_combinations only considers non-empty subsets, so this should be empty.
    let hand = card(Suit::Oros, 12); // value 10
    let table = vec![card(Suit::Copas, 1)]; // value 1 => 10+1=11 != 15
    let combos = Game::find_combinations(&hand, &table);
    assert!(combos.is_empty());
}

// ===========================================================================
// Game play_card tests
// ===========================================================================

#[test]
fn play_card_capture_removes_from_table_and_hand() {
    let mut game = Game::new("Alice", "Bob");
    game.state = GameState::Playing;

    // Set up hand and table manually
    game.players[0].hand = vec![card(Suit::Oros, 7)];         // value 7
    game.table = vec![card(Suit::Copas, 10)];                 // value 8
    game.current_player = 0;

    let result = game.play_card(0, Some(vec![0])).unwrap();
    match result {
        PlayResult::Captured { cards, escoba } => {
            assert_eq!(cards.len(), 2); // table card + hand card
            assert!(escoba); // table was cleared => escoba
        }
        _ => panic!("expected capture"),
    }
    assert!(game.table.is_empty());
    assert!(game.players[0].hand.is_empty());
    assert_eq!(game.players[0].captured.len(), 2);
    assert_eq!(game.players[0].escobas, 1);
}

#[test]
fn play_card_drop_puts_card_on_table() {
    let mut game = Game::new("Alice", "Bob");
    game.state = GameState::Playing;
    game.players[0].hand = vec![card(Suit::Oros, 3)];
    game.table = vec![card(Suit::Copas, 1)];
    game.current_player = 0;

    let result = game.play_card(0, None).unwrap();
    assert_eq!(result, PlayResult::Dropped);
    assert_eq!(game.table.len(), 2);
    assert!(game.players[0].hand.is_empty());
}

#[test]
fn play_card_invalid_combination_returns_error() {
    let mut game = Game::new("Alice", "Bob");
    game.state = GameState::Playing;
    game.players[0].hand = vec![card(Suit::Oros, 1)]; // value 1
    game.table = vec![card(Suit::Copas, 2)];          // value 2 => 1+2=3 != 15
    game.current_player = 0;

    let err = game.play_card(0, Some(vec![0])).unwrap_err();
    assert_eq!(err, GameError::InvalidCombination);
}

#[test]
fn play_card_invalid_hand_index_returns_error() {
    let mut game = Game::new("Alice", "Bob");
    game.state = GameState::Playing;
    game.players[0].hand = vec![card(Suit::Oros, 1)];
    game.current_player = 0;

    let err = game.play_card(5, None).unwrap_err();
    assert_eq!(err, GameError::InvalidCard);
}

#[test]
fn play_card_when_not_playing_returns_error() {
    let mut game = Game::new("Alice", "Bob");
    // state defaults to Dealing
    assert_eq!(game.state, GameState::Dealing);
    game.players[0].hand = vec![card(Suit::Oros, 1)];

    let err = game.play_card(0, None).unwrap_err();
    assert_eq!(err, GameError::GameNotInPlay);
}

#[test]
fn play_card_capture_without_escoba_when_table_not_empty() {
    let mut game = Game::new("Alice", "Bob");
    game.state = GameState::Playing;
    game.players[0].hand = vec![card(Suit::Oros, 7)];                 // value 7
    game.table = vec![card(Suit::Copas, 10), card(Suit::Bastos, 1)]; // value 8, value 1
    game.current_player = 0;

    // Capture index 0 (Sota, value 8): 7+8=15
    let result = game.play_card(0, Some(vec![0])).unwrap();
    match result {
        PlayResult::Captured { escoba, .. } => {
            assert!(!escoba, "table still has cards, no escoba");
        }
        _ => panic!("expected capture"),
    }
    assert_eq!(game.table.len(), 1); // one card remains
}

// ===========================================================================
// Deal round tests
// ===========================================================================

#[test]
fn deal_round_first_deal_gives_3_each_plus_4_table() {
    let mut game = Game::new("Alice", "Bob");
    game.deal_round();
    assert_eq!(game.players[0].hand.len(), 3);
    assert_eq!(game.players[1].hand.len(), 3);
    assert_eq!(game.table.len(), 4);
    assert_eq!(game.deck.remaining(), 40 - 6 - 4); // 30
    assert_eq!(game.state, GameState::Playing);
}

#[test]
fn deal_round_second_deal_gives_3_each_no_table() {
    let mut game = Game::new("Alice", "Bob");
    game.deal_round(); // first: 6 player cards + 4 table
    let table_before = game.table.len();

    // Clear hands to simulate playing them
    game.players[0].hand.clear();
    game.players[1].hand.clear();
    game.deal_round(); // second deal
    assert_eq!(game.players[0].hand.len(), 3);
    assert_eq!(game.players[1].hand.len(), 3);
    assert_eq!(game.table.len(), table_before); // table unchanged
    assert_eq!(game.deck.remaining(), 40 - 6 - 4 - 6); // 24
}

// ===========================================================================
// Scoring tests
// ===========================================================================

#[test]
fn scoring_more_cards_gets_cards_point() {
    let captured = vec![card(Suit::Oros, 1), card(Suit::Copas, 2), card(Suit::Bastos, 3)];
    let opponent = vec![card(Suit::Espadas, 4), card(Suit::Copas, 5)];
    let score = calculate_score(&captured, 0, &opponent);
    assert_eq!(score.cards_point, 1);
}

#[test]
fn scoring_fewer_cards_gets_no_cards_point() {
    let captured = vec![card(Suit::Oros, 1)];
    let opponent = vec![card(Suit::Espadas, 4), card(Suit::Copas, 5)];
    let score = calculate_score(&captured, 0, &opponent);
    assert_eq!(score.cards_point, 0);
}

#[test]
fn scoring_equal_cards_gets_no_point() {
    let captured = vec![card(Suit::Oros, 1), card(Suit::Copas, 2)];
    let opponent = vec![card(Suit::Espadas, 3), card(Suit::Bastos, 4)];
    let score = calculate_score(&captured, 0, &opponent);
    assert_eq!(score.cards_point, 0);
}

#[test]
fn scoring_more_oros_gets_oros_point() {
    let captured = vec![card(Suit::Oros, 1), card(Suit::Oros, 3), card(Suit::Oros, 5)];
    let opponent = vec![card(Suit::Oros, 2)];
    let score = calculate_score(&captured, 0, &opponent);
    assert_eq!(score.oros_point, 1);
}

#[test]
fn scoring_siete_de_velo_gets_point() {
    let captured = vec![card(Suit::Oros, 7)];
    let opponent = vec![card(Suit::Copas, 7)];
    let score = calculate_score(&captured, 0, &opponent);
    assert_eq!(score.siete_velo_point, 1);
}

#[test]
fn scoring_no_siete_de_velo_gets_no_point() {
    let captured = vec![card(Suit::Copas, 7)];
    let opponent = vec![card(Suit::Oros, 7)];
    let score = calculate_score(&captured, 0, &opponent);
    assert_eq!(score.siete_velo_point, 0);
}

#[test]
fn scoring_more_sevens_gets_sevens_point() {
    let captured = vec![card(Suit::Copas, 7), card(Suit::Espadas, 7), card(Suit::Bastos, 7)];
    let opponent = vec![card(Suit::Oros, 7)];
    let score = calculate_score(&captured, 0, &opponent);
    assert_eq!(score.sevens_point, 1);
}

#[test]
fn scoring_escobas_add_to_total() {
    let captured = vec![card(Suit::Copas, 1)];
    let opponent = vec![card(Suit::Bastos, 2), card(Suit::Espadas, 3)];
    let score = calculate_score(&captured, 4, &opponent);
    assert_eq!(score.escobas_points, 4);
    assert!(score.total >= 4);
}

#[test]
fn scoring_total_sums_all_components() {
    // Player has: more cards (1), more oros (1), siete de velo (1), more sevens (1), 2 escobas
    let captured = vec![
        card(Suit::Oros, 7),   // siete de velo + oros + seven
        card(Suit::Oros, 1),   // oros
        card(Suit::Copas, 7),  // seven
        card(Suit::Bastos, 3),
        card(Suit::Espadas, 5),
    ];
    let opponent = vec![card(Suit::Copas, 1), card(Suit::Bastos, 2)];
    let score = calculate_score(&captured, 2, &opponent);
    // cards_point=1, oros_point=1, siete_velo_point=1, sevens_point=1, escobas=2
    assert_eq!(score.total, 6);
}

// ===========================================================================
// is_game_over tests
// ===========================================================================

#[test]
fn game_over_returns_none_when_below_15() {
    let scores = vec![
        ("Alice".to_string(), 10),
        ("Bob".to_string(), 12),
    ];
    assert_eq!(is_game_over(&scores), None);
}

#[test]
fn game_over_returns_winner_when_at_15() {
    let scores = vec![
        ("Alice".to_string(), 15),
        ("Bob".to_string(), 12),
    ];
    assert_eq!(is_game_over(&scores), Some("Alice".to_string()));
}

#[test]
fn game_over_returns_winner_with_higher_score() {
    let scores = vec![
        ("Alice".to_string(), 15),
        ("Bob".to_string(), 17),
    ];
    assert_eq!(is_game_over(&scores), Some("Bob".to_string()));
}

#[test]
fn game_over_tied_at_15_returns_none() {
    let scores = vec![
        ("Alice".to_string(), 15),
        ("Bob".to_string(), 15),
    ];
    assert_eq!(is_game_over(&scores), None);
}

#[test]
fn game_over_tied_above_15_returns_none() {
    let scores = vec![
        ("Alice".to_string(), 18),
        ("Bob".to_string(), 18),
    ];
    assert_eq!(is_game_over(&scores), None);
}

// ===========================================================================
// Player tests
// ===========================================================================

#[test]
fn player_new_starts_empty() {
    let p = Player::new("TestPlayer");
    assert_eq!(p.name, "TestPlayer");
    assert!(p.hand.is_empty());
    assert!(p.captured.is_empty());
    assert_eq!(p.escobas, 0);
}

#[test]
fn player_add_to_hand_and_remove() {
    let mut p = Player::new("P1");
    let c = card(Suit::Oros, 7);
    p.add_to_hand(vec![c]);
    assert_eq!(p.hand.len(), 1);
    let removed = p.remove_from_hand(&c);
    assert!(removed.is_some());
    assert!(p.hand.is_empty());
}

#[test]
fn player_capture_and_record_escoba() {
    let mut p = Player::new("P1");
    p.capture(vec![card(Suit::Copas, 3), card(Suit::Bastos, 5)]);
    assert_eq!(p.captured.len(), 2);
    p.record_escoba();
    p.record_escoba();
    assert_eq!(p.escobas, 2);
}
