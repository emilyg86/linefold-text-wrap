//! Greedy word wrapping for plain text.
//!
//! `wrap` takes text and a column width and returns the text reflowed so
//! that no line exceeds that width, measured in terminal display columns.
//! Blank lines in the input are treated as paragraph breaks and
//! preserved; everything else (single newlines, tabs, runs of spaces) is
//! collapsed to a single space before rewrapping, the same way most text
//! formatters treat "soft" line breaks.
//!
//! `fill` does the same wrapping but pads each line out to the full
//! width, left, right, or center aligned, for callers that want a block
//! of text with a straight edge rather than a ragged one.
//!
//! Width accounts for combining marks (zero columns) and common wide
//! East Asian scripts (two columns) via a hand-picked table, not the
//! full Unicode East Asian Width property, and it doesn't cluster
//! graphemes: a multi-codepoint emoji is measured codepoint by
//! codepoint. Good enough for source comments, commit messages, and
//! terminal output; not a substitute for a Unicode line-breaking
//! algorithm (UAX #14) if you need to handle arbitrary scripts
//! correctly.

mod width;

use width::{display_width, display_width_str};

/// Marks an optional break point inside a word (U+00AD). It's invisible
/// in ordinary text; `wrap_words` treats it as a hint for where to break
/// an overlong word, printing a literal `-` at the point it actually
/// uses and dropping the mark everywhere else.
const SOFT_HYPHEN: char = '\u{00AD}';

/// Wraps `text` to `width` columns, preserving paragraph breaks.
///
/// A width of `0` is treated as `1`, since a line that can hold nothing
/// isn't a useful line and callers passing `0` almost always mean
/// "as narrow as possible" rather than "produce no output".
pub fn wrap(text: &str, width: usize) -> String {
    split_paragraphs(text)
        .iter()
        .map(|p| wrap_words(p, width).join("\n"))
        .collect::<Vec<_>>()
        .join("\n\n")
}

/// How `fill` pads a wrapped line out to the full column width.
///
/// `Left` is wrap's own behavior: no trailing padding, since nothing
/// downstream cares about trailing spaces on a left-aligned line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Alignment {
    Left,
    Right,
    Center,
}

/// Wraps `text` like `wrap`, then pads every line out to `width` columns
/// according to `alignment`.
///
/// Padding is spaces added around the line, not spaces inserted between
/// words, so a line's own word spacing is untouched. A line already at
/// or past `width` (only possible for a hard-broken chunk of an overlong
/// word) is left as-is rather than padded into a negative width.
pub fn fill(text: &str, width: usize, alignment: Alignment) -> String {
    let width = width.max(1);
    split_paragraphs(text)
        .iter()
        .map(|p| {
            wrap_words(p, width)
                .into_iter()
                .map(|line| pad_line(&line, width, alignment))
                .collect::<Vec<_>>()
                .join("\n")
        })
        .collect::<Vec<_>>()
        .join("\n\n")
}

/// Pads `line` to `width` display columns per `alignment`.
fn pad_line(line: &str, width: usize, alignment: Alignment) -> String {
    let line_width = display_width_str(line);
    if line_width >= width {
        return line.to_string();
    }

    let padding = width - line_width;
    match alignment {
        Alignment::Left => line.to_string(),
        Alignment::Right => format!("{}{line}", " ".repeat(padding)),
        Alignment::Center => {
            // The odd column, if any, goes on the right so a centered
            // line that can't split evenly still looks left-leaning
            // rather than drifting right.
            let left = padding / 2;
            let right = padding - left;
            format!("{}{line}{}", " ".repeat(left), " ".repeat(right))
        }
    }
}

/// Wraps a single paragraph (no embedded blank lines) into lines of at
/// most `width` display columns, breaking on whitespace and greedily
/// packing as many words per line as fit.
///
/// A word longer than `width` on its own is broken across multiple
/// lines rather than left overflowing, since silently ignoring the width
/// limit would defeat the point of wrapping. A soft hyphen (U+00AD)
/// already present in the word is used as the preferred break point,
/// with a `-` printed at the line it breaks; a word with no soft
/// hyphens, or one that still doesn't fit between them, falls back to
/// breaking mid-character.
pub fn wrap_words(text: &str, width: usize) -> Vec<String> {
    let width = width.max(1);
    let mut lines = Vec::new();
    let mut current = String::new();
    let mut current_len = 0usize;

    for word in text.split_whitespace() {
        let word_len = display_width_str(word);

        if word_len > width {
            if !current.is_empty() {
                lines.push(std::mem::take(&mut current));
                current_len = 0;
            }
            lines.extend(hyphenate(word, width));
            continue;
        }

        // +1 accounts for the space that would join this word to the
        // current line; there's no such cost when the line is empty.
        let extra = if current.is_empty() { word_len } else { word_len + 1 };
        let word = strip_soft_hyphens(word);

        if current_len + extra > width {
            lines.push(std::mem::take(&mut current));
            current.push_str(&word);
            current_len = word_len;
        } else {
            if !current.is_empty() {
                current.push(' ');
            }
            current.push_str(&word);
            current_len += extra;
        }
    }

    if !current.is_empty() {
        lines.push(current);
    }

    lines
}

/// Drops any soft hyphens from `word`, since one that didn't end up at
/// a line break has no business appearing in the output.
fn strip_soft_hyphens(word: &str) -> std::borrow::Cow<'_, str> {
    if word.contains(SOFT_HYPHEN) {
        std::borrow::Cow::Owned(word.chars().filter(|&c| c != SOFT_HYPHEN).collect())
    } else {
        std::borrow::Cow::Borrowed(word)
    }
}

/// Wraps `text` to `width` columns like `wrap`, then indents every
/// non-blank line by `indent` spaces. Words are packed into
/// `width - indent` columns so the indent plus the wrapped text
/// together never exceed `width`; the blank lines `wrap` uses to mark
/// paragraph breaks are left untouched rather than indented.
///
/// `indent` is clamped to leave at least one column for content, so an
/// indent as wide as or wider than `width` still produces output no
/// wider than `width` instead of overflowing it.
pub fn wrap_indented(text: &str, width: usize, indent: usize) -> String {
    let width = width.max(1);
    let indent = indent.min(width - 1);
    let content_width = width - indent;
    let prefix = " ".repeat(indent);

    wrap(text, content_width)
        .lines()
        .map(|line| {
            if line.is_empty() {
                String::new()
            } else {
                format!("{prefix}{line}")
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Splits `text` into paragraphs, where a paragraph is one or more
/// consecutive non-blank lines joined with single spaces. Runs of one or
/// more blank lines separate paragraphs and collapse to a single break.
fn split_paragraphs(text: &str) -> Vec<String> {
    let mut paragraphs = Vec::new();
    let mut current = String::new();

    for line in text.lines() {
        if line.trim().is_empty() {
            if !current.is_empty() {
                paragraphs.push(std::mem::take(&mut current));
            }
        } else {
            if !current.is_empty() {
                current.push(' ');
            }
            current.push_str(line);
        }
    }

    if !current.is_empty() {
        paragraphs.push(current);
    }

    paragraphs
}

/// Splits an overlong `word` into chunks of at most `width` display
/// columns each, preferring to break at soft hyphens already present in
/// the word over breaking mid-character.
///
/// Each segment between soft hyphens is packed onto the current chunk
/// greedily, the same way `wrap_words` packs words onto a line, except
/// the separator here is a `-` charged only when a break actually lands
/// between two segments, not the segments' natural join. A segment
/// wider than `width` on its own still falls back to `hard_break`,
/// since a soft hyphen can't help split something that has no smaller
/// pieces to offer.
fn hyphenate(word: &str, width: usize) -> Vec<String> {
    if !word.contains(SOFT_HYPHEN) {
        return hard_break(word, width);
    }

    let segments: Vec<&str> = word.split(SOFT_HYPHEN).collect();
    let last = segments.len() - 1;
    let mut chunks = Vec::new();
    let mut current = String::new();
    let mut current_len = 0usize;

    for (i, segment) in segments.iter().copied().enumerate() {
        let segment_len = display_width_str(segment);

        if segment_len > width {
            if !current.is_empty() {
                if current_len < width {
                    current.push('-');
                }
                chunks.push(std::mem::take(&mut current));
                current_len = 0;
            }
            chunks.extend(hard_break(segment, width));
            continue;
        }

        // Reserve a column for a trailing hyphen unless this is the
        // word's last segment, since at this point there's no way to
        // know yet whether another segment will follow it onto the
        // same chunk.
        let reserve = if i == last { 0 } else { 1 };

        if current_len + segment_len + reserve > width && !current.is_empty() {
            // A chunk already at `width` (only possible when `width`
            // is too narrow to fit even one column of hyphen) is left
            // without one rather than pushed over the limit.
            if current_len < width {
                current.push('-');
            }
            chunks.push(std::mem::take(&mut current));
            current_len = 0;
        }

        current.push_str(segment);
        current_len += segment_len;
    }

    if !current.is_empty() {
        chunks.push(current);
    }

    chunks
}

/// Splits a single word into chunks of at most `width` display columns
/// each. Used when a word alone is too long to fit on any line.
///
/// A combining mark never starts a new chunk on its own: since it adds
/// no columns, it stays attached to the character before it even if
/// that pushes the chunk past `width`. A single character wider
/// than `width` (a lone CJK character on a one-column line, say) is
/// left on its own chunk rather than split, since there's no way to
/// divide a character into partial columns.
fn hard_break(word: &str, width: usize) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut chunk = String::new();
    let mut chunk_width = 0usize;

    for ch in word.chars() {
        let ch_width = display_width(ch);
        if chunk_width + ch_width > width && !chunk.is_empty() {
            chunks.push(std::mem::take(&mut chunk));
            chunk_width = 0;
        }
        chunk.push(ch);
        chunk_width += ch_width;
    }

    if !chunk.is_empty() {
        chunks.push(chunk);
    }

    chunks
}
