use vietkb_core::{
    Charset, EngineAction, EngineOptions, InputMethod, KeyInput, SimpleVietnameseEngine,
    VietnameseEngine,
};

const EXPECTED: &str = "Công nghệ trí tuệ nhân tạo đang thay đổi sâu sắc cách chúng ta sống và làm việc mỗi ngày. Từ việc tự động hóa các quy trình phức tạp đến hỗ trợ sáng tạo nghệ thuật, AI mở ra những cánh cửa tri thức mới. Việc làm chủ công nghệ này sẽ giúp con người tối ưu hóa hiệu suất và kiến tạo tương lai.";

const TELEX_INPUT: &str = "Coong ngheej tris tueej nhaan taoj ddang thay ddooir saau sawcs casch chusng ta soongs vaf lamf vieejc mooxi ngayf. Tuwf vieejc tuwj ddoongj hosa casc quy trinfh phuwcs tapj ddeens hoox trowj sasng taoj ngheej thuaajt, AI mowr ra nhuwxng casnh cuwra tri thuwcs mowis. Vieejc lamf chur coong ngheej nayf sex giups con nguwowif toosi uuw hosa hieeju suaats vaf kieens taoj tuwowng lai.";
const VNI_INPUT: &str = "Co6ng nghe65 tri1 tue65 nha6n tao5 d9ang thay d9o6i3 sa6u sa8c1 cach1 chu1ng ta so6ng1 va2 lam2 vie65c mo6i4 ngay2. Tu82 vie65c tu85 d9o6ng5 ho1a cac1 quy trinh2 phu8c1 tap5 d9e6n1 ho64 tro85 sang1 tao5 nghe65 thua6t5, AI mo83 ra nhu8ng4 canh1 cu83a tri thu8c1 mo8i1. Vie65c lam2 chu3 co6ng nghe65 nay2 se4 giup1 con ngu8o8i2 to6i1 u8u ho1a hie6u5 sua6t1 va2 kie6n1 tao5 tu8o8ng lai.";
const VIQR_INPUT: &str = "Co^ng nghe^. tri' tue^. nha^n ta.o d-ang thay d-o^?i sa^u sa('c ca'ch chu'ng ta so^'ng va` lam` vie^.c mo^~i ngay`\\. Tu+` vie^.c tu+. d-o^.ng ho'a ca'c quy tri`nh phu+'c ta.p d-e^'n ho^~ tro+. sa'ng ta.o nghe^. thua^.t, AI mo+? ra nhu+~ng ca'nh cu+?a tri thu+'c mo+'i\\. Vie^.c lam` chu? co^ng nghe^. nay` se~ giup' con ngu+o+`i to^'i u+u ho'a hie^.u sua^'t va` kie^'n ta.o tu+o+ng lai\\.";

fn simulate(method: InputMethod, charset: Charset, input: &str, options: EngineOptions) -> String {
    let mut engine = SimpleVietnameseEngine::new();
    engine.set_input_method(method);
    engine.set_charset(charset);
    engine.set_options(options);
    let mut output = String::new();

    for ch in input.chars() {
        match engine.process_key(KeyInput::from_char(ch)) {
            EngineAction::Noop
            | EngineAction::Preedit(_)
            | EngineAction::ToggleVietnamese
            | EngineAction::BeepOrReject => {}
            EngineAction::Commit(text) => output.push_str(&text),
            EngineAction::Replace {
                delete_graphemes,
                insert,
            } => {
                for _ in 0..delete_graphemes {
                    output.pop();
                }
                output.push_str(&insert);
            }
        }
    }

    output
}

fn method_input(method: InputMethod) -> &'static str {
    match method {
        InputMethod::Telex => TELEX_INPUT,
        InputMethod::Vni => VNI_INPUT,
        InputMethod::Viqr => VIQR_INPUT,
    }
}

/// Settings that do NOT change tone/mark placement for the target paragraph.
/// - `old_oa_uy`: changes oa/oe/uy placement → needs separate input, excluded.
/// - `smart_mark`: changes tone position for ao/oi/ay etc. → needs separate input, excluded.
/// - `use_j_f_w_as_consonants`: only affects j/f/w as consonants, not relevant here.
/// - `spell_check` / `restore_wrong_word` / `skip_english_words`: affect English bypass, not Vietnamese.
fn basic_setting_variants() -> Vec<EngineOptions> {
    let base = EngineOptions::default();
    vec![
        base,
        EngineOptions {
            use_j_f_w_as_consonants: false,
            ..base
        },
        EngineOptions {
            spell_check: false,
            ..base
        },
        EngineOptions {
            restore_wrong_word: true,
            ..base
        },
        EngineOptions {
            skip_english_words: false,
            ..base
        },
        EngineOptions {
            use_j_f_w_as_consonants: false,
            spell_check: false,
            restore_wrong_word: true,
            skip_english_words: false,
            ..base
        },
    ]
}

#[test]
fn paragraph_matches_expected_for_basic_setting_variants() {
    for method in [InputMethod::Telex, InputMethod::Vni, InputMethod::Viqr] {
        for options in basic_setting_variants() {
            let actual = simulate(method, Charset::Unicode, method_input(method), options);
            assert_eq!(actual, EXPECTED, "method={method:?} options={options:?}");
        }
    }
}
