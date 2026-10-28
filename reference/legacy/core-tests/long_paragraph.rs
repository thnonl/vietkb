use vietkb_core::{
    Charset, EngineAction, EngineOptions, InputMethod, KeyInput, SimpleVietnameseEngine,
    VietnameseEngine,
};

const EXPECTED: &str = "Đoạn so sánh tương phản là đoạn văn có sự so sánh trái ngược nhau về nội dung ý tưởng: những hình ảnh thơ văn, phong cách tác giả, hiện thực cuộc sống,…tương phản nhau.\n\nVí dụ 1: Đoạn văn so sánh tương phản, nội dung nói về việc học hành :\n\nTrong cuộc sống, không thiếu những người cho rằng cần học tập để trở thành kẻ có tài, có tri thức giỏi hơn người trước mà không hề nghĩ tới việc rèn luyện đạo đức, lễ nghĩa vốn là giá trị cao quý nhất trong các giá trị của con người( 1). Những người ý luôn hợm mình, không chút khiêm tốn, đôi khi trở thành người vô lễ, có hại cho xã hội(2). Đối với những người ấy, chúng ta cần giúp họ hiểu rõ lời dạy của cổ nhân: “ Tiên học lễ, hậu học văn”( 3).\n\nMô hình đoạn văn: Ý tưởng của đoạn văn là nói về quan niệm của việc học: học để làm người. Câu 1,2 nêu nội dung trái ngược với ý tưởng; câu 3 nêu ý tưởng. Nội dung tương phản với ý tưởng bao giờ cũng được đề cập trước, sau đó dẫn đến nội dung chính của ý tưởng. Đây là đoạn văn mở bài, giải thích câu nói của Khổng Tử “Tiên học lễ, hậu học văn”.\n\nVí dụ 2: Đoạn văn so sánh tương phản, nội dung nói về phẩm chất của con người mới trong “ Lặng lẽ Sa Pa” của Nguyễn Thành Long:\n\nThực lòng mà nói, giữa bao lo toan hối hả của cuộc sống thường ngày, có khi nào ta dành ra được những phút tĩnh lặng của cuộc đời, để lắng nghe nhịp đập bên trong thầm lặng của cuộc sống. Đọc “Lặng lẽ Sa Pa”, ta giật mình bởi những điều Nguyễn Thành Long nói tới mà ta quen nghĩ, quen nhìn hời hợt, nông cạn theo một công thức đã có sẵn mà không chịu đi sâu tìm tòi, phát hiện bản chất bên trong của nó: “ Trong cái lặng im của Sa Pa, dưới những dinh thự cũ kĩ của Sa Pa, Sa Pa mà chỉ nghe tên, người ta đã nghĩ đến chuyện nghỉ ngơi, có những con người làm việc và suy nghĩ” hết mình cho đất nước, cho cuộc sống hôm nay.";

#[derive(Clone, Copy)]
struct VietnameseChar {
    base: char,
    mark: Option<char>,
    tone: Option<char>,
}

fn decompose(ch: char) -> Option<VietnameseChar> {
    let (base, mark, tone) = match ch {
        'á' => ('a', None, Some('s')),
        'à' => ('a', None, Some('f')),
        'ả' => ('a', None, Some('r')),
        'ã' => ('a', None, Some('x')),
        'ạ' => ('a', None, Some('j')),
        'ắ' => ('a', Some('w'), Some('s')),
        'ằ' => ('a', Some('w'), Some('f')),
        'ẳ' => ('a', Some('w'), Some('r')),
        'ẵ' => ('a', Some('w'), Some('x')),
        'ặ' => ('a', Some('w'), Some('j')),
        'ă' => ('a', Some('w'), None),
        'ấ' => ('a', Some('a'), Some('s')),
        'ầ' => ('a', Some('a'), Some('f')),
        'ẩ' => ('a', Some('a'), Some('r')),
        'ẫ' => ('a', Some('a'), Some('x')),
        'ậ' => ('a', Some('a'), Some('j')),
        'â' => ('a', Some('a'), None),
        'é' => ('e', None, Some('s')),
        'è' => ('e', None, Some('f')),
        'ẻ' => ('e', None, Some('r')),
        'ẽ' => ('e', None, Some('x')),
        'ẹ' => ('e', None, Some('j')),
        'ế' => ('e', Some('e'), Some('s')),
        'ề' => ('e', Some('e'), Some('f')),
        'ể' => ('e', Some('e'), Some('r')),
        'ễ' => ('e', Some('e'), Some('x')),
        'ệ' => ('e', Some('e'), Some('j')),
        'ê' => ('e', Some('e'), None),
        'í' => ('i', None, Some('s')),
        'ì' => ('i', None, Some('f')),
        'ỉ' => ('i', None, Some('r')),
        'ĩ' => ('i', None, Some('x')),
        'ị' => ('i', None, Some('j')),
        'ó' => ('o', None, Some('s')),
        'ò' => ('o', None, Some('f')),
        'ỏ' => ('o', None, Some('r')),
        'õ' => ('o', None, Some('x')),
        'ọ' => ('o', None, Some('j')),
        'ố' => ('o', Some('o'), Some('s')),
        'ồ' => ('o', Some('o'), Some('f')),
        'ổ' => ('o', Some('o'), Some('r')),
        'ỗ' => ('o', Some('o'), Some('x')),
        'ộ' => ('o', Some('o'), Some('j')),
        'ô' => ('o', Some('o'), None),
        'ớ' => ('o', Some('w'), Some('s')),
        'ờ' => ('o', Some('w'), Some('f')),
        'ở' => ('o', Some('w'), Some('r')),
        'ỡ' => ('o', Some('w'), Some('x')),
        'ợ' => ('o', Some('w'), Some('j')),
        'ơ' => ('o', Some('w'), None),
        'ú' => ('u', None, Some('s')),
        'ù' => ('u', None, Some('f')),
        'ủ' => ('u', None, Some('r')),
        'ũ' => ('u', None, Some('x')),
        'ụ' => ('u', None, Some('j')),
        'ứ' => ('u', Some('w'), Some('s')),
        'ừ' => ('u', Some('w'), Some('f')),
        'ử' => ('u', Some('w'), Some('r')),
        'ữ' => ('u', Some('w'), Some('x')),
        'ự' => ('u', Some('w'), Some('j')),
        'ư' => ('u', Some('w'), None),
        'ý' => ('y', None, Some('s')),
        'ỳ' => ('y', None, Some('f')),
        'ỷ' => ('y', None, Some('r')),
        'ỹ' => ('y', None, Some('x')),
        'ỵ' => ('y', None, Some('j')),
        'đ' => ('d', Some('d'), None),
        'Á' => ('A', None, Some('s')),
        'À' => ('A', None, Some('f')),
        'Ả' => ('A', None, Some('r')),
        'Ã' => ('A', None, Some('x')),
        'Ạ' => ('A', None, Some('j')),
        'Ắ' => ('A', Some('w'), Some('s')),
        'Ằ' => ('A', Some('w'), Some('f')),
        'Ẳ' => ('A', Some('w'), Some('r')),
        'Ẵ' => ('A', Some('w'), Some('x')),
        'Ặ' => ('A', Some('w'), Some('j')),
        'Ă' => ('A', Some('w'), None),
        'Ấ' => ('A', Some('a'), Some('s')),
        'Ầ' => ('A', Some('a'), Some('f')),
        'Ẩ' => ('A', Some('a'), Some('r')),
        'Ẫ' => ('A', Some('a'), Some('x')),
        'Ậ' => ('A', Some('a'), Some('j')),
        'Â' => ('A', Some('a'), None),
        'É' => ('E', None, Some('s')),
        'È' => ('E', None, Some('f')),
        'Ẻ' => ('E', None, Some('r')),
        'Ẽ' => ('E', None, Some('x')),
        'Ẹ' => ('E', None, Some('j')),
        'Ế' => ('E', Some('e'), Some('s')),
        'Ề' => ('E', Some('e'), Some('f')),
        'Ể' => ('E', Some('e'), Some('r')),
        'Ễ' => ('E', Some('e'), Some('x')),
        'Ệ' => ('E', Some('e'), Some('j')),
        'Ê' => ('E', Some('e'), None),
        'Í' => ('I', None, Some('s')),
        'Ì' => ('I', None, Some('f')),
        'Ỉ' => ('I', None, Some('r')),
        'Ĩ' => ('I', None, Some('x')),
        'Ị' => ('I', None, Some('j')),
        'Ó' => ('O', None, Some('s')),
        'Ò' => ('O', None, Some('f')),
        'Ỏ' => ('O', None, Some('r')),
        'Õ' => ('O', None, Some('x')),
        'Ọ' => ('O', None, Some('j')),
        'Ố' => ('O', Some('o'), Some('s')),
        'Ồ' => ('O', Some('o'), Some('f')),
        'Ổ' => ('O', Some('o'), Some('r')),
        'Ỗ' => ('O', Some('o'), Some('x')),
        'Ộ' => ('O', Some('o'), Some('j')),
        'Ô' => ('O', Some('o'), None),
        'Ớ' => ('O', Some('w'), Some('s')),
        'Ờ' => ('O', Some('w'), Some('f')),
        'Ở' => ('O', Some('w'), Some('r')),
        'Ỡ' => ('O', Some('w'), Some('x')),
        'Ợ' => ('O', Some('w'), Some('j')),
        'Ơ' => ('O', Some('w'), None),
        'Ú' => ('U', None, Some('s')),
        'Ù' => ('U', None, Some('f')),
        'Ủ' => ('U', None, Some('r')),
        'Ũ' => ('U', None, Some('x')),
        'Ụ' => ('U', None, Some('j')),
        'Ứ' => ('U', Some('w'), Some('s')),
        'Ừ' => ('U', Some('w'), Some('f')),
        'Ử' => ('U', Some('w'), Some('r')),
        'Ữ' => ('U', Some('w'), Some('x')),
        'Ự' => ('U', Some('w'), Some('j')),
        'Ư' => ('U', Some('w'), None),
        'Ý' => ('Y', None, Some('s')),
        'Ỳ' => ('Y', None, Some('f')),
        'Ỷ' => ('Y', None, Some('r')),
        'Ỹ' => ('Y', None, Some('x')),
        'Ỵ' => ('Y', None, Some('j')),
        'Đ' => ('D', Some('d'), None),
        _ => return None,
    };
    Some(VietnameseChar { base, mark, tone })
}

fn encode_word(word: &str, method: InputMethod) -> String {
    let mut out = String::new();
    let mut tone = None;

    for ch in word.chars() {
        if let Some(vietnamese) = decompose(ch) {
            out.push(vietnamese.base);
            if let Some(mark) = vietnamese.mark {
                match method {
                    InputMethod::Telex => out.push(mark),
                    InputMethod::Vni => out.push(match mark {
                        'a' | 'e' | 'o' => '6',
                        'w' => '8',
                        'd' => '9',
                        _ => unreachable!(),
                    }),
                    InputMethod::Viqr => unreachable!(),
                }
            }
            tone = tone.or(vietnamese.tone);
        } else {
            out.push(ch);
        }
    }

    if let Some(tone) = tone {
        out.push(match method {
            InputMethod::Telex => tone,
            InputMethod::Vni => match tone {
                's' => '1',
                'f' => '2',
                'r' => '3',
                'x' => '4',
                'j' => '5',
                _ => unreachable!(),
            },
            InputMethod::Viqr => unreachable!(),
        });
    }

    out
}

fn encode_text(text: &str, method: InputMethod) -> String {
    let mut out = String::new();
    let mut word = String::new();

    for ch in text.chars() {
        if ch.is_alphabetic() || decompose(ch).is_some() {
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

fn simulate(method: InputMethod, input: &str) -> String {
    let mut engine = SimpleVietnameseEngine::new();
    engine.set_input_method(method);
    engine.set_charset(Charset::Unicode);
    engine.set_options(EngineOptions::default());
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

#[test]
fn long_paragraph_matches_expected_for_telex_and_vni() {
    for method in [InputMethod::Telex, InputMethod::Vni] {
        let input = encode_text(EXPECTED, method);
        let actual = simulate(method, &input);
        assert_eq!(actual, EXPECTED, "method={method:?}");
    }
}
