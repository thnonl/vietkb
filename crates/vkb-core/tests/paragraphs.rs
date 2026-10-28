//! Golden paragraphs. Each word is encoded mechanically from the expected text
//! (marks inline, tone at the end of the syllable) and typed back through the
//! engine, so a placement regression anywhere in the paragraph names the word
//! that broke.

use vkb_core::letters::{decompose, Mark, Tone};
use vkb_core::{simulate, EngineOptions, InputMethod};

const VIETNAMESE: &str = "Công nghệ trí tuệ nhân tạo đang thay đổi sâu sắc cách chúng ta sống và làm việc mỗi ngày. \
Từ việc tự động hóa các quy trình phức tạp đến hỗ trợ sáng tạo nghệ thuật, AI mở ra những cánh cửa tri thức mới. \
Việc làm chủ công nghệ này giúp con người tối ưu hóa hiệu suất và kiến tạo tương lai.";

const MIXED: &str = "Microsoft và Google dẫn đầu cuộc đua công nghệ toàn cầu. Docker giúp deployment lên cloud \
nhanh hơn, JavaScript và Python vẫn thống trị. GitHub ghi nhận mọi nỗ lực của bạn.";

fn tone_key(method: InputMethod, tone: Tone) -> char {
    match method {
        InputMethod::Telex => match tone {
            Tone::Acute => 's',
            Tone::Grave => 'f',
            Tone::Hook => 'r',
            Tone::Tilde => 'x',
            Tone::Dot => 'j',
            Tone::None => unreachable!(),
        },
        InputMethod::Vni => match tone {
            Tone::Acute => '1',
            Tone::Grave => '2',
            Tone::Hook => '3',
            Tone::Tilde => '4',
            Tone::Dot => '5',
            Tone::None => unreachable!(),
        },
        InputMethod::Legacy => unreachable!("the removed-slug sentinel is never simulated"),
    }
}

fn encode_word(word: &str, method: InputMethod) -> String {
    let mut out = String::new();
    let mut tone = Tone::None;
    for ch in word.chars() {
        let (base, mark, letter_tone) = match decompose(ch) {
            Some(parts) => parts,
            None => {
                out.push(ch);
                continue;
            }
        };
        let upper = ch.is_uppercase();
        let base_char = if upper {
            base.to_ascii_uppercase()
        } else {
            base
        };
        out.push(base_char);
        match (method, mark) {
            (_, Mark::None) => {}
            (InputMethod::Telex, Mark::Breve) => out.push('w'),
            (InputMethod::Telex, Mark::Circumflex) => out.push(base_char),
            (InputMethod::Telex, Mark::Horn) => out.push('w'),
            (InputMethod::Telex, Mark::Bar) => out.push(if upper { 'D' } else { 'd' }),
            (InputMethod::Vni, Mark::Breve) => out.push('8'),
            (InputMethod::Vni, Mark::Circumflex) => out.push('6'),
            (InputMethod::Vni, Mark::Horn) => out.push('7'),
            (InputMethod::Vni, Mark::Bar) => out.push('9'),
            (InputMethod::Legacy, _) => unreachable!("the removed-slug sentinel is never simulated"),
        }
        if tone == Tone::None {
            tone = letter_tone;
        }
    }
    if tone != Tone::None {
        out.push(tone_key(method, tone));
    }
    out
}

fn words(text: &str) -> impl Iterator<Item = &str> {
    text.split(|ch: char| !ch.is_alphabetic())
        .filter(|word| !word.is_empty())
}

fn encode_text(text: &str, method: InputMethod) -> String {
    let mut out = String::new();
    let mut word = String::new();
    for ch in text.chars() {
        if ch.is_alphabetic() {
            word.push(ch);
        } else {
            if !word.is_empty() {
                out.push_str(&encode_word(&word, method));
                word.clear();
            }
            out.push(ch);
        }
    }
    if !word.is_empty() {
        out.push_str(&encode_word(&word, method));
    }
    out
}

#[test]
fn vietnamese_paragraph_round_trips() {
    for method in [InputMethod::Telex, InputMethod::Vni] {
        let options = EngineOptions {
            method,
            ..EngineOptions::default()
        };
        let input = encode_text(VIETNAMESE, method);
        assert_eq!(simulate(options, &input), VIETNAMESE, "method {method:?}");
    }
}

#[test]
fn every_vietnamese_word_is_correct_on_its_own() {
    for method in [InputMethod::Telex, InputMethod::Vni] {
        let options = EngineOptions {
            method,
            ..EngineOptions::default()
        };
        for word in words(VIETNAMESE) {
            let input = encode_word(word, method);
            assert_eq!(
                simulate(options, &input),
                word,
                "method {method:?} word {word:?} typed as {input:?}"
            );
        }
    }
}

#[test]
fn mixed_english_paragraph_keeps_english_words_raw() {
    for method in [InputMethod::Telex, InputMethod::Vni] {
        let options = EngineOptions {
            method,
            ..EngineOptions::default()
        };
        for word in words(MIXED) {
            let input = encode_word(word, method);
            assert_eq!(
                simulate(options, &input),
                word,
                "method {method:?} word {word:?} typed as {input:?}"
            );
        }
    }
}

#[test]
fn chained_words_do_not_leak_state() {
    // Same word twice in a row must produce the accent twice.
    let options = EngineOptions::default();
    assert_eq!(simulate(options, "tieengs tieengs"), "tiếng tiếng");
    assert_eq!(simulate(options, "hoaf hoaf hoaf"), "hòa hòa hòa");
    assert_eq!(simulate(options, "quas quas"), "quá quá");
}
