mod strings;

use escoba15_engine::*;
use std::io::{self, BufRead, Write};
use std::process;
use strings::Strings;

enum GameMode {
    HumanVsHuman,
    HumanVsAi(Difficulty),
}

enum Action {
    Back,
    Drop,
    Capture(usize),
}

enum Input {
    Number(usize),
    Back,
    Invalid,
}

fn main() {
    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();

    println!("========================================");
    println!("        ESCOBA DEL 15");
    println!("========================================");
    println!();

    // Language selection (always shown in both languages)
    println!("  1. Espanol");
    println!("  2. English");
    let lang = loop {
        let choice = prompt_usize_raw(&mut lines, "Idioma / Language [1-2]: ");
        match choice {
            1 => break Lang::Es,
            2 => break Lang::En,
            _ => println!("1 o 2 / 1 or 2"),
        }
    };

    let s = strings::for_lang(lang);
    println!();
    println!("  {}", s.quit_hint);
    println!();

    // Game mode selection
    println!("{}:", s.game_mode);
    println!("  1. {}", s.mode_hvh);
    println!("  2. {}", s.mode_ai_easy);
    println!("  3. {}", s.mode_ai_medium);
    println!("  4. {}", s.mode_ai_hard);
    let mode = loop {
        let choice = prompt_usize(&mut lines, s.choose_mode, s);
        match choice {
            1 => break GameMode::HumanVsHuman,
            2 => break GameMode::HumanVsAi(Difficulty::Easy),
            3 => break GameMode::HumanVsAi(Difficulty::Medium),
            4 => break GameMode::HumanVsAi(Difficulty::Hard),
            _ => println!("1-4."),
        }
    };
    println!();

    let default_p1 = match lang { Lang::Es => "Jugador 1", Lang::En => "Player 1" };
    let default_p2 = match lang { Lang::Es => "Jugador 2", Lang::En => "Player 2" };
    let default_player = match lang { Lang::Es => "Jugador", Lang::En => "Player" };

    let (p1_name, p2_name) = match &mode {
        GameMode::HumanVsHuman => {
            let p1 = prompt_name(&mut lines, s.player_1_prompt, default_p1, s);
            let p2 = prompt_name(&mut lines, s.player_2_prompt, default_p2, s);
            (p1, p2)
        }
        GameMode::HumanVsAi(diff) => {
            let p1 = prompt_name(&mut lines, s.your_name_prompt, default_player, s);
            let ai_name = match diff {
                Difficulty::Easy => s.ai_easy,
                Difficulty::Medium => s.ai_medium,
                Difficulty::Hard => s.ai_hard,
            };
            (p1, ai_name.to_string())
        }
    };

    println!();
    println!("{} {} {}", p1_name, s.vs, p2_name);
    println!("{}", s.first_to_15);

    let mut cumulative_scores: [(String, u32); 2] = [
        (p1_name.clone(), 0),
        (p2_name.clone(), 0),
    ];
    let mut round_number = 0u32;

    loop {
        round_number += 1;
        println!();
        println!("========================================");
        println!("  {} {}", s.round, round_number);
        println!("========================================");

        let mut game = Game::new(&p1_name, &p2_name);
        game.deal_round();

        while game.state == GameState::Playing {
            match &mode {
                GameMode::HumanVsAi(diff) if game.current_player == 1 => {
                    play_ai_turn(&mut game, *diff, lang, s);
                }
                _ => {
                    play_human_turn(&mut game, &mut lines, lang, s);
                }
            }
        }

        println!();
        println!("----------------------------------------");
        println!("  {} {}", s.round_results, round_number);
        println!("----------------------------------------");

        let s0 = calculate_score(
            &game.players[0].captured,
            game.players[0].escobas,
            &game.players[1].captured,
        );
        let s1 = calculate_score(
            &game.players[1].captured,
            game.players[1].escobas,
            &game.players[0].captured,
        );

        print_score_breakdown(&game.players[0].name, &s0, s);
        print_score_breakdown(&game.players[1].name, &s1, s);

        cumulative_scores[0].1 += s0.total;
        cumulative_scores[1].1 += s1.total;

        println!();
        println!("  {} {} = {} pts, {} = {} pts",
            s.cumulative,
            cumulative_scores[0].0, cumulative_scores[0].1,
            cumulative_scores[1].0, cumulative_scores[1].1,
        );

        if let Some(winner) = is_game_over(&cumulative_scores) {
            println!();
            println!("========================================");
            println!("  {} {}", winner.to_uppercase(), s.wins_game);
            println!("========================================");
            break;
        }
    }
}

fn play_ai_turn(game: &mut Game, difficulty: Difficulty, lang: Lang, s: &Strings) {
    let ai_name = game.players[game.current_player].name.clone();

    println!();
    println!("----------------------------------------");
    println!("  {}", s.turn.replace("{}", &ai_name));
    println!("----------------------------------------");

    println!();
    print_table(&game.table, lang, s);

    println!();
    print!("{} {} ", ai_name, s.thinking);
    io::stdout().flush().ok();

    let ai_move = suggest_play(game, difficulty);

    println!("... done!");

    let hand_card = game.players[game.current_player].hand[ai_move.hand_index];
    println!("{} {} {}", ai_name, s.plays, hand_card.localized_name(lang));

    match game.play_card(ai_move.hand_index, ai_move.table_indices) {
        Ok(PlayResult::Captured { cards, escoba }) => {
            let names: Vec<String> = cards.iter().map(|c| c.localized_name(lang)).collect();
            println!("{}", s.captured.replace("{}", &names.join(", ")));
            if escoba {
                println!();
                println!("{}", s.swept_table.replace("{}", &ai_name));
            }
        }
        Ok(PlayResult::Dropped) => {
            println!("{}", s.drops.replace("{}", &hand_card.localized_name(lang))
                .replace("{}", &ai_name));
        }
        Err(e) => {
            println!("AI error: {:?}", e);
        }
    }

    game.next_turn();
}

fn play_human_turn(game: &mut Game, lines: &mut io::Lines<io::StdinLock<'_>>, lang: Lang, s: &Strings) {
    let player_name = game.players[game.current_player].name.clone();

    println!();
    println!("----------------------------------------");
    println!("  {}", s.turn.replace("{}", &player_name));
    println!("----------------------------------------");

    println!();
    print_table(&game.table, lang, s);

    let (hand_idx, capture_indices) = loop {
        let plays = game.valid_plays();

        println!();
        println!("{}", s.your_hand);
        for play in &plays {
            let has_captures = if play.captures.is_empty() { "" } else { " *" };
            println!("  {}. {} (val:{}){}",
                play.hand_index + 1,
                play.hand_card.localized_name(lang),
                play.hand_card.value(),
                has_captures,
            );
        }

        let hand_choice = loop {
            let choice = prompt_usize(lines, s.pick_card, s);
            if choice >= 1 && choice <= plays.len() {
                break choice - 1;
            }
            println!("{} 1-{}.", s.invalid_choice, plays.len());
        };

        let play = &plays[hand_choice];

        if play.captures.is_empty() {
            println!();
            println!("{}", s.no_captures.replace("{}", &play.hand_card.localized_name(lang)));
            break (play.hand_index, None);
        }

        println!();
        println!("{}",
            s.possible_captures
                .replace("{}", &play.hand_card.localized_name(lang))
                .replacen("{}", &play.hand_card.value().to_string(), 1)
        );
        for (i, capture) in play.captures.iter().enumerate() {
            let desc: Vec<String> = capture
                .table_cards
                .iter()
                .map(|c| format!("{} (val:{})", c.localized_name(lang), c.value()))
                .collect();
            println!("  {}. {}", i + 1, desc.join(" + "));
        }
        println!("  0. {}", s.drop_option);
        println!("  b. {}", s.back_option);

        let num_captures = play.captures.len();
        let action = loop {
            match prompt_input(lines, s.choose_action, s) {
                Input::Number(0) => break Action::Drop,
                Input::Number(n) if n >= 1 && n <= num_captures => break Action::Capture(n),
                Input::Back => break Action::Back,
                _ => println!("{} 0-{} / 'b'.", s.invalid_choice, num_captures),
            }
        };

        match action {
            Action::Back => {
                println!("{}", s.going_back);
                continue;
            }
            Action::Drop => break (play.hand_index, None),
            Action::Capture(n) => {
                let indices = play.captures[n - 1].table_indices.clone();
                break (play.hand_index, Some(indices));
            }
        }
    };

    match game.play_card(hand_idx, capture_indices) {
        Ok(PlayResult::Captured { cards, escoba }) => {
            let names: Vec<String> = cards.iter().map(|c| c.localized_name(lang)).collect();
            println!("{}", s.captured.replace("{}", &names.join(", ")));
            if escoba {
                println!();
                println!("{}", s.escoba);
            }
        }
        Ok(PlayResult::Dropped) => {
            println!("{}", s.card_dropped);
        }
        Err(e) => {
            println!("Error: {:?}", e);
            return;
        }
    }

    game.next_turn();
}

fn print_table(table: &[Card], lang: Lang, s: &Strings) {
    if table.is_empty() {
        println!("{}", s.table_empty);
    } else {
        println!("{}", s.table_cards);
        for (i, card) in table.iter().enumerate() {
            println!("  {}. {} (val:{})", i + 1, card.localized_name(lang), card.value());
        }
    }
}

fn print_score_breakdown(name: &str, score: &ScoreBreakdown, s: &Strings) {
    println!();
    println!("  {}:", name);
    println!("    {:<17} {} {}", s.score_cards, score.cards_point, if score.cards_point > 0 { "<-" } else { "" });
    println!("    {:<17} {} {}", s.score_oros, score.oros_point, if score.oros_point > 0 { "<-" } else { "" });
    println!("    {:<17} {} {}", s.score_siete_velo, score.siete_velo_point, if score.siete_velo_point > 0 { "<-" } else { "" });
    println!("    {:<17} {} {}", s.score_sevens, score.sevens_point, if score.sevens_point > 0 { "<-" } else { "" });
    println!("    {:<17} {}", s.score_escobas, score.escobas_points);
    println!("    -----------------");
    println!("    {:<17} {}", s.score_round_total, score.total);
}

// --- Input helpers ---

fn read_line(lines: &mut io::Lines<io::StdinLock<'_>>, prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().ok();
    let input = lines
        .next()
        .and_then(|r| r.ok())
        .unwrap_or_default()
        .trim()
        .to_string();
    input
}

fn check_quit(input: &str, s: &Strings) {
    let lower = input.to_lowercase();
    if lower == "q" || lower == "quit" || lower == "exit" || lower == "salir" {
        println!();
        println!("{}", s.goodbye);
        process::exit(0);
    }
}

fn prompt_name(
    lines: &mut io::Lines<io::StdinLock<'_>>,
    prompt: &str,
    default: &str,
    s: &Strings,
) -> String {
    let input = read_line(lines, prompt);
    check_quit(&input, s);
    if input.is_empty() {
        default.to_string()
    } else {
        input
    }
}

/// Used before language is selected (no Strings available yet)
fn prompt_usize_raw(lines: &mut io::Lines<io::StdinLock<'_>>, prompt: &str) -> usize {
    loop {
        let input = read_line(lines, prompt);
        let lower = input.to_lowercase();
        if lower == "q" || lower == "quit" || lower == "exit" {
            println!();
            println!("Bye! / Chau!");
            process::exit(0);
        }
        if let Ok(n) = input.parse::<usize>() {
            return n;
        }
    }
}

fn prompt_usize(lines: &mut io::Lines<io::StdinLock<'_>>, prompt: &str, s: &Strings) -> usize {
    loop {
        let input = read_line(lines, prompt);
        check_quit(&input, s);
        if let Ok(n) = input.parse::<usize>() {
            return n;
        }
        println!("{}", s.enter_number);
    }
}

fn prompt_input(lines: &mut io::Lines<io::StdinLock<'_>>, prompt: &str, s: &Strings) -> Input {
    let input = read_line(lines, prompt);
    check_quit(&input, s);
    let lower = input.to_lowercase();
    if lower == "b" || lower == "back" || lower == "volver" {
        return Input::Back;
    }
    match input.parse::<usize>() {
        Ok(n) => Input::Number(n),
        Err(_) => Input::Invalid,
    }
}
