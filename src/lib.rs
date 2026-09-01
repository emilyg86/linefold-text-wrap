//! Greedy word wrapping for plain text.
//!
//! `wrap` takes text and a column width and returns the text reflowed so
//! that no line exceeds that width, measured in characters. Blank lines in
//! the input are treated as paragraph breaks and preserved; everything else
//! (single newlines, tabs, runs of spaces) is collapsed to a single space
//! before rewrapping, the same way most text formatters treat "soft" line
//! breaks.
//!
//! Width is counted in `char`s, not display columns, so this will
//! misjudge combining marks and double-width glyphs (CJK, most emoji).
//! Good enough for source comments, commit messages, and terminal output
//! in a Latin alphabet; not a substitute for a Unicode line-breaking
//! algorithm (UAX #14) if you need to handle arbitrary scripts correctly.

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

/// Wraps a single paragraph (no embedded blank lines) into lines of at
/// most `width` characters, breaking on whitespace and greedily packing
/// as many words per line as fit.
///
/// A word longer than `width` on its own is hard-broken across multiple
/// lines rather than left overflowing, since silently ignoring the width
/// limit would defeat the point of wrapping.
pub fn wrap_words(text: &str, width: usize) -> Vec<String> {
    let width = width.max(1);
    let mut lines = Vec::new();
    let mut current = String::new();
    let mut current_len = 0usize;

    for word in text.split_whitespace() {
        let word_len = word.chars().count();

        if word_len > width {
            if !current.is_empty() {
                lines.push(std::mem::take(&mut current));
                current_len = 0;
            }
            lines.extend(hard_break(word, width));
            continue;
        }

        // +1 accounts for the space that would join this word to the
        // current line; there's no such cost when the line is empty.
        let extra = if current.is_empty() { word_len } else { word_len + 1 };

        if current_len + extra > width {
            lines.push(std::mem::take(&mut current));
            current.push_str(word);
            current_len = word_len;
        } else {
            if !current.is_empty() {
                current.push(' ');
            }
            current.push_str(word);
            current_len += extra;
        }
    }

    if !current.is_empty() {
        lines.push(current);
    }

    lines
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

/// Splits a single word into chunks of at most `width` characters each.
/// Used when a word alone is too long to fit on any line.
fn hard_break(word: &str, width: usize) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut chunk = String::new();
    let mut count = 0usize;

    for ch in word.chars() {
        if count == width {
            chunks.push(std::mem::take(&mut chunk));
            count = 0;
        }
        chunk.push(ch);
        count += 1;
    }

    if !chunk.is_empty() {
        chunks.push(chunk);
    }

    chunks
}
