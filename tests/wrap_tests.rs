use linefold::{fill, wrap, wrap_indented, Alignment};

struct Case {
    name: &'static str,
    input: &'static str,
    width: usize,
    expected: &'static str,
}

#[test]
fn wrap_table() {
    let cases = vec![
        Case {
            name: "empty input",
            input: "",
            width: 10,
            expected: "",
        },
        Case {
            name: "whitespace only collapses to nothing",
            input: "   \n\t  ",
            width: 10,
            expected: "",
        },
        Case {
            name: "single short word is untouched",
            input: "hello",
            width: 10,
            expected: "hello",
        },
        Case {
            name: "word exactly at width fits on one line",
            input: "hello",
            width: 5,
            expected: "hello",
        },
        Case {
            name: "word one char longer than width is hard-broken",
            input: "hello",
            width: 4,
            expected: "hell\no",
        },
        Case {
            name: "runs of spaces collapse to one",
            input: "a    b",
            width: 10,
            expected: "a b",
        },
        Case {
            name: "tabs and single newlines collapse like spaces",
            input: "a\tb\nc",
            width: 10,
            expected: "a b c",
        },
        Case {
            name: "leading and trailing whitespace is dropped",
            input: "  hi there  ",
            width: 10,
            expected: "hi there",
        },
        Case {
            name: "greedy fill packs as many words as fit",
            input: "one two three",
            width: 7,
            expected: "one two\nthree",
        },
        Case {
            name: "an overlong word breaks mid-paragraph without eating neighbors",
            input: "a supercalifragilistic word",
            width: 6,
            expected: "a\nsuperc\nalifra\ngilist\nic\nword",
        },
        Case {
            name: "a blank line marks a paragraph break",
            input: "first paragraph\n\nsecond paragraph",
            width: 20,
            expected: "first paragraph\n\nsecond paragraph",
        },
        Case {
            name: "several blank lines collapse to a single paragraph break",
            input: "first\n\n\n\nsecond",
            width: 20,
            expected: "first\n\nsecond",
        },
        Case {
            name: "multibyte characters are counted per char, not per byte",
            input: "café résumé",
            width: 6,
            expected: "café\nrésumé",
        },
        Case {
            name: "width of one puts one character per line",
            input: "abc",
            width: 1,
            expected: "a\nb\nc",
        },
        Case {
            name: "width of zero is treated as width of one",
            input: "abc",
            width: 0,
            expected: "a\nb\nc",
        },
        Case {
            name: "wide CJK characters count as two columns each",
            input: "こんにちは",
            width: 6,
            expected: "こんに\nちは",
        },
        Case {
            name: "a wide character wider than the width gets its own line",
            input: "中 ab",
            width: 1,
            expected: "中\na\nb",
        },
        Case {
            name: "combining marks add no width of their own",
            input: "e\u{0301}e\u{0301}e\u{0301}",
            width: 3,
            expected: "e\u{0301}e\u{0301}e\u{0301}",
        },
        Case {
            name: "a hard break keeps a combining mark with its base character",
            input: "e\u{0301}e\u{0301}e\u{0301}e\u{0301}",
            width: 2,
            expected: "e\u{0301}e\u{0301}\ne\u{0301}e\u{0301}",
        },
        Case {
            name: "an overlong word breaks at a soft hyphen instead of mid-character",
            input: "auto\u{00AD}mobile",
            width: 6,
            expected: "auto-\nmobile",
        },
        Case {
            name: "a soft hyphen in a word that already fits is dropped, not printed",
            input: "auto\u{00AD}mobile",
            width: 20,
            expected: "automobile",
        },
        Case {
            name: "a soft hyphen break is kept even when the next segment still \
                   needs a hard break",
            input: "un\u{00AD}bearable",
            width: 3,
            expected: "un-\nbea\nrab\nle",
        },
    ];

    let mut failures = Vec::new();
    for case in &cases {
        let got = wrap(case.input, case.width);
        if got != case.expected {
            failures.push(format!(
                "case '{}' failed:\n  input:    {:?}\n  width:    {}\n  expected: {:?}\n  got:      {:?}",
                case.name, case.input, case.width, case.expected, got
            ));
        }
    }

    assert!(failures.is_empty(), "\n{}", failures.join("\n\n"));
}

struct IndentCase {
    name: &'static str,
    input: &'static str,
    width: usize,
    indent: usize,
    expected: &'static str,
}

#[test]
fn wrap_indented_table() {
    let cases = vec![
        IndentCase {
            name: "indent is prepended to every packed line",
            input: "one two three four",
            width: 10,
            indent: 2,
            expected: "  one two\n  three\n  four",
        },
        IndentCase {
            name: "paragraph-break blank lines stay unindented",
            input: "first paragraph\n\nsecond paragraph",
            width: 20,
            indent: 2,
            expected: "  first paragraph\n\n  second paragraph",
        },
        IndentCase {
            name: "zero indent behaves exactly like wrap",
            input: "a b c",
            width: 3,
            indent: 0,
            expected: "a b\nc",
        },
        IndentCase {
            name: "an indent as wide as the width is clamped, not overflowed",
            input: "ab",
            width: 3,
            indent: 10,
            expected: "  a\n  b",
        },
        IndentCase {
            name: "empty input stays empty regardless of indent",
            input: "",
            width: 10,
            indent: 4,
            expected: "",
        },
    ];

    let mut failures = Vec::new();
    for case in &cases {
        let got = wrap_indented(case.input, case.width, case.indent);
        if got != case.expected {
            failures.push(format!(
                "case '{}' failed:\n  input:    {:?}\n  width:    {}\n  indent:   {}\n  expected: {:?}\n  got:      {:?}",
                case.name, case.input, case.width, case.indent, case.expected, got
            ));
        }
    }

    assert!(failures.is_empty(), "\n{}", failures.join("\n\n"));
}

struct FillCase {
    name: &'static str,
    input: &'static str,
    width: usize,
    alignment: Alignment,
    expected: &'static str,
}

#[test]
fn fill_table() {
    let cases = vec![
        FillCase {
            name: "left alignment matches wrap exactly",
            input: "one two three",
            width: 7,
            alignment: Alignment::Left,
            expected: "one two\nthree",
        },
        FillCase {
            name: "right alignment pads each line on the left",
            input: "one two three",
            width: 7,
            alignment: Alignment::Right,
            expected: "one two\n  three",
        },
        FillCase {
            name: "center alignment splits padding evenly on both sides",
            input: "one two three",
            width: 9,
            alignment: Alignment::Center,
            expected: " one two \n  three  ",
        },
        FillCase {
            name: "a line already at width gets no padding",
            input: "hello",
            width: 5,
            alignment: Alignment::Right,
            expected: "hello",
        },
        FillCase {
            name: "odd padding puts the extra column on the right",
            input: "hello",
            width: 4,
            alignment: Alignment::Center,
            expected: "hell\n o  ",
        },
        FillCase {
            name: "paragraph breaks stay unpadded between paragraphs",
            input: "hi\n\nbye",
            width: 6,
            alignment: Alignment::Right,
            expected: "    hi\n\n   bye",
        },
        FillCase {
            name: "empty input stays empty regardless of alignment",
            input: "",
            width: 10,
            alignment: Alignment::Center,
            expected: "",
        },
        FillCase {
            name: "wide CJK characters count toward padding by display width",
            input: "中 a",
            width: 6,
            alignment: Alignment::Right,
            expected: "  中 a",
        },
    ];

    let mut failures = Vec::new();
    for case in &cases {
        let got = fill(case.input, case.width, case.alignment);
        if got != case.expected {
            failures.push(format!(
                "case '{}' failed:\n  input:    {:?}\n  width:    {}\n  alignment: {:?}\n  expected: {:?}\n  got:      {:?}",
                case.name, case.input, case.width, case.alignment, case.expected, got
            ));
        }
    }

    assert!(failures.is_empty(), "\n{}", failures.join("\n\n"));
}
