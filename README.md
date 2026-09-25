# linefold

A small library for wrapping plain text to a fixed column width, plus a
one-argument CLI on top of it.

Most of the time text wrapping is trivial, and then you hit a word longer
than the line width, or a blank line that's supposed to mean "new
paragraph" instead of "just another space", and the naive
`split(' ').chunks(width)` approach falls apart. This is the version of
that logic that handles those cases on purpose instead of by accident.

## What it does

- Greedily packs words onto each line, breaking on whitespace.
- Treats blank lines as paragraph breaks and preserves them; everything
  else (single newlines, tabs, repeated spaces) is treated as ordinary
  whitespace and collapsed.
- Hard-breaks a word that's longer than the requested width on its own,
  instead of leaving a line that overflows. If the word already has a
  soft hyphen (U+00AD) marking where it may be split, that's used as the
  break point and rendered as a `-`; a soft hyphen that isn't needed is
  dropped rather than left in the output.
- Counts width in terminal display columns, not raw characters: a
  combining mark adds no width and common wide East Asian scripts (CJK
  ideographs, Hangul, fullwidth forms) count as two columns. The table
  behind this is hand-picked, not a full copy of the Unicode East Asian
  Width property, and it doesn't cluster graphemes, so an emoji built
  from several codepoints is still measured codepoint by codepoint.

## Library usage

```rust
fn main() {
    let text = "This is a sentence that is a bit too long to fit on one \
                narrow terminal line.";

    println!("{}", linefold::wrap(text, 20));
}
```

```
This is a sentence
that is a bit too
long to fit on one
narrow terminal
line.
```

`wrap` handles a whole block of text, including paragraph breaks. If you
already have a single paragraph split into lines yourself, `wrap_words`
skips the paragraph-splitting step and returns a `Vec<String>` of the
wrapped lines directly.

`wrap_indented` wraps like `wrap` and then indents every line by a
fixed number of spaces, narrowing the packed text so the indent never
pushes a line past the requested width:

```rust
println!("{}", linefold::wrap_indented("a short note", 12, 2));
```

```
  a short
  note
```

`fill` wraps like `wrap` but also pads every line out to the full width,
left, right, or center aligned:

```rust
use linefold::Alignment;

println!("{}", linefold::fill("one two three", 7, Alignment::Right));
```

```
one two
  three
```

## CLI usage

The binary reads text from stdin and writes the wrapped result to
stdout:

```sh
cat notes.txt | linefold 60
```

Width defaults to 80 columns if omitted:

```sh
echo "some text to wrap" | linefold
```

Pass `--indent N` to indent every output line by `N` spaces:

```sh
echo "some text to wrap" | linefold 60 --indent 4
```

## Building

No third-party dependencies, so a plain `cargo build` or `cargo test` is
all that's needed.

## License

MIT, see LICENSE.
