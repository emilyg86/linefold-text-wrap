//! Display-column width for a single `char`.
//!
//! Terminals render most characters in one column, but a combining
//! mark takes no column of its own (it modifies the glyph before it)
//! and a large chunk of East Asian scripts render two columns wide.
//! This is a hand-picked table of the Unicode blocks that matter in
//! practice, not a generated copy of the full Unicode East Asian
//! Width or Combining Character properties, so it will get obscure
//! characters wrong. It also doesn't attempt grapheme clustering: an
//! emoji built from a zero-width-joiner sequence is measured joiner
//! by joiner, not as the single glyph a terminal renders it as.

/// Returns how many terminal columns `ch` occupies: `0` for a
/// combining mark, `2` for a wide East Asian character, `1` otherwise.
pub(crate) fn display_width(ch: char) -> usize {
    let cp = ch as u32;

    if is_combining(cp) {
        0
    } else if is_wide(cp) {
        2
    } else {
        1
    }
}

/// Sums the display width of every character in `s`.
pub(crate) fn display_width_str(s: &str) -> usize {
    s.chars().map(display_width).sum()
}

fn is_combining(cp: u32) -> bool {
    matches!(cp,
        0x0300..=0x036F   // Combining Diacritical Marks
        | 0x0483..=0x0489 // Combining Cyrillic
        | 0x0591..=0x05BD // Hebrew points
        | 0x05BF | 0x05C1 | 0x05C2 | 0x05C4 | 0x05C5 | 0x05C7
        | 0x0610..=0x061A // Arabic marks
        | 0x064B..=0x065F // Arabic combining marks
        | 0x0670
        | 0x06D6..=0x06DC
        | 0x06DF..=0x06E4
        | 0x0730..=0x074A // Syriac
        | 0x07A6..=0x07B0 // Thaana
        | 0x0900..=0x0903 // Devanagari signs
        | 0x093A..=0x093C
        | 0x0941..=0x0948
        | 0x0951..=0x0957
        | 0x0AC1..=0x0AC8
        | 0x1AB0..=0x1AFF // Combining Diacritical Marks Extended
        | 0x1DC0..=0x1DFF // Combining Diacritical Marks Supplement
        | 0x20D0..=0x20FF // Combining Diacritical Marks for Symbols
        | 0xFE20..=0xFE2F // Combining Half Marks
        | 0x200B          // zero width space
        | 0x200C          // zero width non-joiner
        | 0x200D          // zero width joiner
        | 0xFEFF          // zero width no-break space / BOM
    )
}

fn is_wide(cp: u32) -> bool {
    matches!(cp,
        0x1100..=0x115F     // Hangul Jamo
        | 0x2E80..=0x303E   // CJK Radicals .. CJK Symbols and Punctuation
        | 0x3041..=0x33FF   // Hiragana .. CJK Compatibility
        | 0x3400..=0x4DBF   // CJK Unified Ideographs Extension A
        | 0x4E00..=0x9FFF   // CJK Unified Ideographs
        | 0xA000..=0xA4CF   // Yi Syllables, Yi Radicals
        | 0xAC00..=0xD7A3   // Hangul Syllables
        | 0xF900..=0xFAFF   // CJK Compatibility Ideographs
        | 0xFE30..=0xFE4F   // CJK Compatibility Forms
        | 0xFF00..=0xFF60   // Fullwidth Forms
        | 0xFFE0..=0xFFE6   // Fullwidth Signs
        | 0x20000..=0x2FFFD // CJK Unified Ideographs Extension B and beyond
        | 0x30000..=0x3FFFD
    )
}

#[cfg(test)]
mod tests {
    use super::{display_width, display_width_str};

    #[test]
    fn ascii_is_one_column() {
        assert_eq!(display_width('a'), 1);
        assert_eq!(display_width('Z'), 1);
        assert_eq!(display_width(' '), 1);
    }

    #[test]
    fn combining_marks_are_zero_columns() {
        assert_eq!(display_width('\u{0301}'), 0); // combining acute accent
        assert_eq!(display_width('\u{200D}'), 0); // zero width joiner
    }

    #[test]
    fn cjk_characters_are_two_columns() {
        assert_eq!(display_width('中'), 2);
        assert_eq!(display_width('あ'), 2);
        assert_eq!(display_width('한'), 2);
        assert_eq!(display_width('！'), 2); // fullwidth punctuation
    }

    #[test]
    fn string_width_sums_its_characters() {
        assert_eq!(display_width_str("中文"), 4);
        assert_eq!(display_width_str("e\u{0301}"), 1); // e + combining acute
        assert_eq!(display_width_str(""), 0);
    }
}
