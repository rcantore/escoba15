# Escoba del 15

> **[Version en espanol](README.es.md)**

A card game engine for [Escoba del 15](https://en.wikipedia.org/wiki/Escoba), the classic trick-taking game played across Spain and Latin America. Built in Rust with an AI opponent that uses Monte Carlo simulations to play — no hardcoded rules, no `if/else` trees, just thousands of simulated games per move.

## What is Escoba del 15?

Two players take turns playing cards from their hand. The goal is to capture cards from the table by finding combinations that **sum to 15** with the card you play. Clear the entire table in one capture and you score an **escoba** (sweep). At the end of each round, points are awarded for:

| Category | Point |
|----------|-------|
| Most cards captured | 1 |
| Most *oros* (coins) captured | 1 |
| The 7 de Oros (*siete de velo*) | 1 |
| Most sevens captured | 1 |
| Each escoba (sweep) | 1 |

First player to reach **15 points** across rounds wins the game.

The deck is a traditional **Spanish 40-card deck**: 4 suits (Oros, Copas, Espadas, Bastos), ranks 1-7 and 10-12 (no 8 or 9).

## Project Structure

```
escoba15/
├── engine/          # Core game library (Rust)
│   ├── src/
│   │   ├── card.rs      # Card model (suit, rank, values)
│   │   ├── deck.rs      # 40-card Spanish deck
│   │   ├── player.rs    # Player state (hand, captures, escobas)
│   │   ├── game.rs      # Game logic, combination finding, turn management
│   │   ├── scoring.rs   # Round scoring and game-over detection
│   │   ├── ai.rs        # MCTS-based AI opponent
│   │   ├── lang.rs      # Multilingual support (Spanish/English)
│   │   ├── wasm.rs      # WebAssembly bridge (optional)
│   │   └── lib.rs       # Public API re-exports
│   └── tests/
│       └── engine_tests.rs  # 49 integration tests
├── cli/             # Terminal interface
│   └── src/
│       ├── main.rs      # Interactive game loop
│       └── strings.rs   # Localized UI strings
└── Cargo.toml       # Workspace config
```

## How the AI Works

The AI doesn't have a list of "if this card, then play that card" rules. Instead, it uses **Information Set Monte Carlo Tree Search (ISMCTS)** — a technique from game AI research designed for games with hidden information.

Here's what happens every time the AI needs to make a move:

### 1. The Problem: Hidden Information

Escoba is an **imperfect information** game. The AI can see its own hand and the table, but it doesn't know:
- What cards the opponent is holding
- What order the remaining deck is in

This means it can't just calculate the "perfect" move like you could in chess. It has to reason under uncertainty.

### 2. Determinization: Imagining Possible Worlds

For each simulation, the AI takes all the cards it **can't** see (opponent's hand + remaining deck) and **shuffles them randomly**. Then it deals them back: the right number go to the opponent's hand, the rest become the deck.

This creates one possible version of reality — a "what if the cards were arranged like *this*?" scenario. It's called **determinization**.

```
What the AI knows:          What the AI imagines:
┌─────────────────┐         ┌─────────────────┐
│ My hand: 3,7,R  │         │ My hand: 3,7,R  │  (same)
│ Table: 5,S      │         │ Table: 5,S      │  (same)
│ Their hand: ???  │   →    │ Their hand: 2,6,C│  (random guess)
│ Deck: ?????????  │         │ Deck: 1,4,A,B...│  (shuffled)
└─────────────────┘         └─────────────────┘
```

### 3. Playout: Simulating to the End

In each imagined world, the AI tries a specific move (say, "play the 7 to capture the table 5+3"), and then **both players play randomly** until the round ends. This is called a **random playout**. It's fast because no thinking is involved — both sides just pick legal moves at random.

### 4. Scoring: Who Won?

After the playout finishes, the AI counts the score: who captured more cards? More oros? The siete de velo? Escobas? It records the result as a win, loss, or draw.

### 5. Repeat Thousands of Times

The AI does this across all its possible moves. For each move, it runs many simulations, and the move with the **highest win rate** is the one it plays.

The **difficulty dial** controls how many simulations run:

| Difficulty | Simulations | Thinking Time |
|-----------|-------------|---------------|
| Easy      | 100         | Instant       |
| Medium    | 1,000       | Fast          |
| Hard      | 10,000      | ~1 second     |

More simulations = better statistical confidence = stronger play. The hard AI genuinely considers the strategic value of each move across thousands of random futures.

### Why This Approach?

- **No hand-tuned heuristics.** The AI discovers what's good by simulating, not by someone programming "prefer oros" rules.
- **Scales with compute.** Want a stronger AI? Just increase the simulation count.
- **Handles uncertainty naturally.** Determinization is a proven technique for imperfect information games.
- **It actually works.** On hard difficulty, the AI makes surprisingly strong strategic decisions — prioritizing escobas, collecting oros, and protecting key cards.

## Combination Finding

Another interesting piece: finding which table cards sum to 15 with your hand card. The engine uses **bitmask subset enumeration** — it generates all 2^n subsets of the table cards using bitwise operations and checks which ones hit the target sum. With a max of ~10 cards on the table, this is at most 1024 subsets — instant.

```rust
for mask in 1..(1u32 << n) {
    let subset_sum = (0..n)
        .filter(|bit| mask & (1 << bit) != 0)
        .map(|bit| table[bit].value())
        .sum();
    if hand_value + subset_sum == 15 {
        // Found a valid capture!
    }
}
```

## Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) (1.70+)

### Build and Test

```bash
# Run all tests (69 tests: 20 unit + 49 integration)
cargo test

# Build the engine
cargo build --release

# Play in the terminal
cargo run --release -p escoba15-cli
```

### Build for WebAssembly

The engine compiles to WASM for browser-based UIs:

```bash
# Install wasm-pack
cargo install wasm-pack

# Build the WASM package
cd engine
wasm-pack build --target web --features wasm
```

This produces a `pkg/` directory with `.wasm` + JS bindings ready to import from any web framework.

## Using the Engine as a Library

```rust
use escoba15_engine::*;

// Create a new game
let mut game = Game::new("Alice", "Bob");
game.deal_round();

// Get valid plays for the current player
let plays = game.valid_plays();

// Play a card (capture table cards at indices 0 and 2)
let result = game.play_card(0, Some(vec![0, 2]));

// Or drop a card (no capture)
let result = game.play_card(1, None);

// Advance to next turn
game.next_turn();

// Ask the AI for a move
let ai_move = suggest_play(&game, Difficulty::Hard);

// Calculate scores at round end
let scores = calculate_score(
    &game.players[0].captured,
    game.players[0].escobas,
    &game.players[1].captured,
);
```

## Multilingual

The engine supports Spanish and English natively:

```rust
use escoba15_engine::{Card, Suit, Lang};

let card = Card::new(Suit::Oros, 12);
card.localized_name(Lang::Es); // "Rey de Oros"
card.localized_name(Lang::En); // "King of Coins"
```

## Test Coverage

69 tests covering:
- Card creation, values, and edge cases
- Deck operations (shuffle, draw, empty)
- Combination finding (empty table, no match, single match, multiple matches, three-card combos)
- Game play (capture, drop, escoba detection, invalid moves, round dealing)
- Scoring (cards, oros, siete de velo, sevens, escobas, totals)
- Game-over detection (threshold, ties, highest score wins)
- AI (valid moves, capture preference, determinization integrity, playout completion, difficulty scaling)

```bash
cargo test
# running 69 tests ... test result: ok. 69 passed
```

## License

MIT
