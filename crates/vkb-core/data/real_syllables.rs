// Hand-curated for VietKB (MIT). Not generated: edit by hand, review in tests/fixtures/d10_not_vietnamese.txt.
//
// Real Vietnamese syllables (the lexicon.rs syllable key: letters, then ^ ( + marks, D for d-bar, tone digit
// last) that the syllable corpus in vi.txt does not attest but that standard Vietnamese words use
// (son1 = són, lai3 = lải, ma3 = mả, cho+ = chơ ...). A listed English word whose typed keys
// compose a syllable NOT in this list is English (room -> rôm stays room). Owner principle, 2026-11-02:
// a word comes out Vietnamese only if the composed text is a real Vietnamese word; doubt means Vietnamese.
// Entries marked U are doubtful (doubt -> real); P are pinned Vietnamese by a test (tons, pins, tongs).
// Compound-only syllables (trê, tủn, kên, vảy) are NOT here: alone they stay English (see the fixture).
// Sorted by byte order (binary search). Keep the list sorted.
pub static REAL_SYLLABLES: &[&str] = &[
    "a(n1", // U
    "a^1", // U
    "a^3", // U
    "ba^", // U
    "bang1",
    "bap1",
    "be^t1",
    "bem1",
    "bem3",
    "bia1", // U
    "bo+1",
    "bo^m", // U
    "bot1", // U
    "bu3", // U
    "bua3",
    "bum2",
    "ca(", // U
    "cha1", // U
    "chai1",
    "cham1", // U
    "chat1",
    "che1",
    "chip1",
    "chit1",
    "cho+",
    "chum1",
    "chun1", // U
    "co^p1",
    "coi1",
    "coi3", // P
    "con3",
    "cop1", // U
    "dai1",
    "de1", // U
    "de^m", // U
    "doc1",
    "dong1",
    "duo1", // U
    "e3", // U
    "e^1",
    "ga1",
    "gang1", // U
    "go+n1", // U
    "go^1", // U
    "go^2",
    "gum1", // U
    "hang1",
    "he^1", // U
    "he^3",
    "hem1", // U
    "hen1", // U
    "heo3",
    "hi1",
    "hip1",
    "ho+",
    "ho+1",
    "ho1", // U
    "hoay1",
    "hop1",
    "hum1", // U
    "i2",
    "im1", // U
    "kit1", // U
    "lai3",
    "lam1", // U
    "lap1",
    "le^1", // U
    "le^3",
    "le^ch1",
    "lia1",
    "lie^t1", // U
    "lo+1",
    "lo4",
    "lo^m1",
    "lo^n",
    "loa1",
    "loi1",
    "ma(", // U
    "ma3",
    "mam1", // U
    "me1",
    "me^1", // U
    "mi1",
    "mo^1",
    "mot1",
    "na^n", // U
    "na^n1", // U
    "nap1", // U
    "ni1", // U
    "ni4", // U
    "no+1", // U
    "no3",
    "nu1", // U
    "nun1", // U
    "o+1",
    "o+n1",
    "o3", // U
    "o4", // U
    "oa1", // U
    "oai1",
    "oat1",
    "oe1",
    "om1", // U
    "op1", // U
    "pa1", // U
    "pan1", // U
    "pe1", // U
    "pe^", // U
    "peo1", // U
    "pet1", // U
    "pin1", // P
    "po+",
    "po+1", // U
    "ram1",
    "re1",
    "re^1", // U
    "re^2",
    "re^t1",
    "rem1", // U
    "rep1", // U
    "ro+1", // U
    "run1", // U
    "rung1",
    "sa1",
    "sa4", // U
    "sa^n1",
    "sa^n3",
    "sac1",
    "sai1",
    "san1",
    "se1", // U
    "se2", // U
    "se^n1",
    "sen1", // U
    "si1", // U
    "sim1", // U
    "sin1", // U
    "sip1",
    "sit1",
    "so+1",
    "son1",
    "su1", // U
    "su2", // U
    "sun1",
    "te^3",
    "te^m",
    "tem1",
    "ten1", // U
    "the1", // U
    "thoa1",
    "thon1", // U
    "thong1", // U
    "to1", // U
    "ton1", // P
    "tong1", // P
    "top1", // U
    "tot1",
    "tram1",
    "tro+1",
    "un1", // U
    "un3",
    "ve^1",
    "veo1",
    "xeo1",
    "xeo4", // U
];
