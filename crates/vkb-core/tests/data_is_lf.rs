//! The embedded tables and the test fixtures must be LF (`.gitattributes`): a CR
//! at the end of a line breaks the run-time table lookups on a Windows checkout.

#[test]
fn embedded_data_and_fixtures_have_no_carriage_return() {
    let files: [(&str, &str); 7] = [
        ("data/english.txt", include_str!("../data/english.txt")),
        ("data/vi.txt", include_str!("../data/vi.txt")),
        ("data/rimes.txt", include_str!("../data/rimes.txt")),
        ("data/consts.rs", include_str!("../data/consts.rs")),
        ("tests/data/tech_mixed_vi.src.txt", include_str!("data/tech_mixed_vi.src.txt")),
        ("tests/data/tech_mixed_vi.txt", include_str!("data/tech_mixed_vi.txt")),
        ("tests/data/tech_mixed_vi_vni.txt", include_str!("data/tech_mixed_vi_vni.txt")),
    ];
    for (name, text) in files {
        assert!(!text.contains('\r'), "{name} has a carriage return: check .gitattributes and core.autocrlf");
    }
}
