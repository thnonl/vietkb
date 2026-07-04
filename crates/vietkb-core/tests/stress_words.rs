use vietkb_core::{
    Charset, EngineAction, EngineOptions, InputMethod, KeyInput, SimpleVietnameseEngine,
    VietnameseEngine,
};

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
fn stress_words_from_very_long_paragraph_match_for_telex_and_vni() {
    let expected = "Đoạn văn là gì? Cách viết đoạn văn. HƯỚNG DẪN CHUNG. Khái niệm phổ thông hiện nay được hiểu theo nhiều cách khác nhau. Tùy theo người đọc cảm nhận mà phân chia ra thành các đoạn. Cách hiểu này khiến cho cách phân đoạn thiếu tính khách quan. Diện mạo đoạn văn không được xác định. Việc xây dựng đoạn văn trở nên khó khăn, phức tạp, khó rèn luyện các thao tác để trở thành kĩ năng kĩ xảo. Đoạn văn xét theo cả hai tiêu chí về ý và về lời. Logic ngữ nghĩa, vai trò chức năng riêng, trật tự nhất định, khía cạnh khác nhau. Thỏa đáng, thuận lợi, rành mạch. Ví dụ về đoạn văn: Vì ông lão yêu làng tha thiết nên vô cùng căm uất khi nghe tin dân làng theo giặc. Ông Hai dứt khoát lựa chọn theo cách của ông. Đây là một nét mới trong tình cảm của người nông dân thời kì đánh Pháp. Chủ đề, khái quát, quy nạp, diễn dịch, tổng phân hợp, nhân quả, vấn đáp, đòn bẩy, giả thiết, móc xích. Sáng tác thơ là một công việc rất đặc biệt, rất khó khăn, đòi hỏi người nghệ sĩ phải hình thành một cá tính sáng tạo. Xuân Diệu, Chính Hữu, Đồng chí, Quang Trung, Nguyễn Huệ, Nguyễn Dữ, Vũ Nương, Nam Xương, thủy cung, Hoàng Giang, Trương Sinh, Phan Lang, Linh Phi, chiếc thoa vàng. Núi Thái Sơn, Trung Quốc, tinh khiết, ngọt ngào, vô tận, chữ hiếu. Kiều, lưu lạc, cát dập sóng vùi, quạt nồng ấp lạnh, sân lai, gốc tử. Ông đồ, Vũ Đình Liên, Hà Nội, xót xa, thanh cao, lãng quên, chôn vùi. Truyện Kiều, Nguyễn Du, mùa xuân, cỏ non xanh rợn chân trời, cành lê trắng điểm một vài bông hoa. Phương thảo liên thiên bích, Lê chi sổ điểm hoa. Cành hoa lê, cô thiếu nữ, hội họa, hòa quyện, dào dạt. Cái bóng, Chuyện người con gái Nam Xương, oan trái khổ đau, bất hạnh, phụ nữ, phong kiến. Trồng cây gây rừng, cây xanh bóng mát, môi trường sống được bảo vệ. Đọc hiểu tác phẩm, hoàn cảnh ra đời, nhan đề tác phẩm, chi tiết quan trọng, từ ngữ đặc sắc, hiệu quả nghệ thuật, biện pháp tu từ. Sang thu, Hữu Thỉnh, cuối năm 1977, Từ chiến hào tới thành phố. Hoàng Lê nhất thống chí, Ngô gia văn phái, Phú Xuân, Rạch Gầm, Xoài Mút, thủy chiến, Đàng Ngoài, Ngọc Hân công chúa, Đống Đa. Mùa xuân nho nhỏ, Thanh Hải, tình yêu cuộc sống. Bài thơ về tiểu đội xe không kính, Phạm Tiến Duật, Trường Sơn, Vầng trăng - Quầng lửa. Chế Lan Viên, Con cò, đồng dao, lời ru ngân nga. Đoàn thuyền đánh cá, Huy Cận, Hòn Gai, Quảng Ninh, miền Bắc. Làng, Kim Lân, Chợ Dầu, tản cư, Việt gian, cải chính. Lặng lẽ Sa Pa, Nguyễn Thành Long, Yên Sơn, khí tượng, vật lí địa cầu. Những ngôi sao xa xôi, Lê Minh Khuê, Phương Định, thanh niên xung phong. Bến quê, Nguyễn Minh Châu, Nhĩ, bãi bồi bên kia sông Hồng. Mặt trời của bắp, mặt trời của mẹ, em nằm trên lưng. Mưa ngừng, gió lùa mau khô thôi. Chỉ cần trong xe có một trái tim. Ơi con chim chiền chiện, hót chi mà vang trời, từng giọt long lanh rơi. Một mùa xuân nho nhỏ, lặng lẽ dâng cho đời. Ôi hàng tre xanh xanh Việt Nam, bão táp mưa sa đứng thẳng hàng. Ánh trăng im phăng phắc đủ cho ta giật mình. Việt Bắc, Tố Hữu, thủy chung. Nguyễn Khoa Điềm, Khúc hát ru những em bé ngủ trên lưng mẹ. Y Phương, Nói với con. Viễn Phương, Viếng lăng Bác. Thúy Kiều, Mã Giám Sinh, Lục Vân Tiên, Kiều Nguyệt Nga.";

    for method in [InputMethod::Telex, InputMethod::Vni] {
        let input = encode_text(expected, method);
        let actual = simulate(method, &input);
        assert_eq!(actual, expected, "method={method:?}");
    }
}
