use vkb_core::{apply_action, Charset, Engine, EngineOptions, InputMethod, KeyInput, VietnameseEngine};

fn simulate(options: EngineOptions, input: &str) -> String {
    simulate_keys(options, &input.chars().map(KeyInput::char).collect::<Vec<_>>())
}

fn simulate_keys(options: EngineOptions, keys: &[KeyInput]) -> String {
    let mut engine = Engine::new(options);
    let mut out = String::new();
    for key in keys {
        let action = engine.process_key(*key);
        apply_action(&mut out, action, *key);
    }
    out
}

struct Session { engine: Engine, out: String }
impl Session {
    fn new(options: EngineOptions) -> Self { Self { engine: Engine::new(options), out: String::new() } }
    fn text(&mut self, input: &str) -> &str {
        for ch in input.chars() {
            let key = KeyInput::char(ch);
            let action = self.engine.process_key(key);
            apply_action(&mut self.out, action, key);
        }
        &self.out
    }
}

fn opts(method: InputMethod) -> EngineOptions { EngineOptions { method, ..EngineOptions::default() } }

#[test] fn modern_tone_telex() {
    assert_eq!(simulate(EngineOptions { modern_tone:true, ..opts(InputMethod::Telex) }, "hoaf"), "hòa");
    assert_eq!(simulate(EngineOptions { modern_tone:false, ..opts(InputMethod::Telex) }, "hoaf"), "hoà");
}
#[test] fn modern_tone_vni() {
    assert_eq!(simulate(EngineOptions { modern_tone:true, ..opts(InputMethod::Vni) }, "hoa2"), "hòa");
    assert_eq!(simulate(EngineOptions { modern_tone:true, ..opts(InputMethod::Vni) }, "hoa1"), "hóa");
    assert_eq!(simulate(EngineOptions { modern_tone:false, ..opts(InputMethod::Vni) }, "hoa2"), "hoà");
    assert_eq!(simulate(EngineOptions { modern_tone:false, ..opts(InputMethod::Vni) }, "hoa1"), "hoá");
}
#[test] fn free_tone_telex() {
    assert_eq!(simulate(EngineOptions { free_tone:true, ..opts(InputMethod::Telex) }, "hoafn"), "hoàn");
    assert_eq!(simulate(EngineOptions { free_tone:false, ..opts(InputMethod::Telex) }, "hoafn"), "hòan");
}
#[test] fn free_tone_vni() {
    // free tone allows tone after final n; with false, mark hit vowel before final n.
    assert_eq!(simulate(EngineOptions { free_tone:true, ..opts(InputMethod::Vni) }, "hoan2"), "hoàn");
    assert_eq!(simulate(EngineOptions { free_tone:false, ..opts(InputMethod::Vni) }, "hoa2n"), "hòan");
}
#[test] fn jfwz_as_consonant_telex() {
    let on = EngineOptions { jfwz_as_consonant:true, ..opts(InputMethod::Telex) };
    let off = EngineOptions { jfwz_as_consonant: false, ..opts(InputMethod::Telex) };
    assert_eq!(simulate(on, "w"), "w");
    assert_eq!(simulate(on, "tw"), "tw");
    // corpus_telex.rs:208 proves `f` remains tone key with default flag.
    assert_eq!(simulate(on, "af"), "à");
    assert_eq!(simulate(off, "w"), "ư");
    assert_eq!(simulate(off, "tw"), "tư");
    assert_eq!(simulate(off, "af"), "à");
}
#[test] fn jfwz_as_consonant_vni() {
    // VNI has no letter-tone keys; this flag has no effect on VNI.
    let input="af";
    assert_eq!(simulate(EngineOptions { jfwz_as_consonant:true, ..opts(InputMethod::Vni) }, input), simulate(EngineOptions { jfwz_as_consonant:false, ..opts(InputMethod::Vni) }, input));
}
#[test] fn auto_capitalize_telex() {
    assert_eq!(simulate(EngineOptions { auto_capitalize:true, ..opts(InputMethod::Telex) }, "tieengs. vieetj"), "tiếng. Việt");
    assert_eq!(simulate(opts(InputMethod::Telex), "tieengs. vieetj"), "tiếng. việt");
}
#[test] fn auto_capitalize_vni() {
    assert_eq!(simulate(EngineOptions { auto_capitalize:true, ..opts(InputMethod::Vni) }, "tie6ng1. vie6t5"), "tiếng. Việt");
    assert_eq!(simulate(opts(InputMethod::Vni), "tie6ng1. vie6t5"), "tiếng. việt");
}
#[test] fn spell_check_telex() {
    // With check on, invalid syllable leaves tone trigger literal; off transforms it.
    assert_eq!(simulate(opts(InputMethod::Telex), "asd"), "asd");
    assert_eq!(simulate(EngineOptions { spell_check:false, ..opts(InputMethod::Telex) }, "asd"), "ád");
}
#[test] fn spell_check_vni() {
    // With check on, invalid syllable leaves trigger literal; off applies VNI tone.
    assert_eq!(simulate(opts(InputMethod::Vni), "a1d"), "a1d");
    assert_eq!(simulate(EngineOptions { spell_check:false, ..opts(InputMethod::Vni) }, "a1d"), "ád");
}
#[test] fn restore_wrong_word_telex() {
    // D4 in 28-engine-rulings.md: spell_check=false disables restore_wrong_word.
    let mut on=Session::new(EngineOptions { restore_wrong_word:true, spell_check:false, ..opts(InputMethod::Telex) });
    assert_eq!(on.text("google "), "gôgle ");
    let mut off=Session::new(EngineOptions { spell_check:false, ..opts(InputMethod::Telex) });
    assert_eq!(off.text("google "), "gôgle ");
}
#[test] fn restore_wrong_word_vni() {
    // With spell check off, both flags allow VNI transforms at boundary.
    let mut on=Session::new(EngineOptions { restore_wrong_word:true, spell_check:false, ..opts(InputMethod::Vni) });
    assert_eq!(on.text("go6gle "), "gôgle ");
    let mut off=Session::new(EngineOptions { spell_check:false, ..opts(InputMethod::Vni) });
    assert_eq!(off.text("go6gle "), "gôgle ");
}
#[test] fn charset_telex() {
    let nfc=simulate(opts(InputMethod::Telex), "tieengs");
    let nfd=simulate(EngineOptions { charset:Charset::UnicodeNfd, ..opts(InputMethod::Telex) }, "tieengs");
    assert_eq!(nfc.chars().count(), 5);
    assert_eq!(nfd.chars().count(), 7);
}
#[test] fn charset_vni() {
    let nfc=simulate(opts(InputMethod::Vni), "tie6ng1");
    let nfd=simulate(EngineOptions { charset:Charset::UnicodeNfd, ..opts(InputMethod::Vni) }, "tie6ng1");
    assert_eq!(nfc.chars().count(), 5);
    assert_eq!(nfd.chars().count(), 7);
}
