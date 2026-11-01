//! Data-generation helper: type each word read from stdin through the Telex
//! engine (English restore OFF) and print `raw<TAB>composed`.
//! Used by `scripts/gen_wordlists.py`; not part of the shipped app.

use std::io::BufRead;

use vkb_core::{simulate, EngineOptions};

fn main() {
    let options = EngineOptions {
        auto_restore_english: false,
        jfwz_as_consonant: std::env::var_os("COMPOSE_NO_JFWZ").is_none(),
        spell_check: std::env::var_os("COMPOSE_NO_SPELL").is_none(),
        ..EngineOptions::default()
    };
    for line in std::io::stdin().lock().lines() {
        let word = line.unwrap_or_default();
        let word = word.trim();
        if word.is_empty() {
            continue;
        }
        let typed = simulate(options, &format!("{word} "));
        println!("{word}\t{}", typed.trim_end());
    }
}
