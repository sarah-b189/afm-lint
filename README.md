# afm-lint

A linter for Adobe Font Metrics (`.afm`) files.

AFM is the plain-text sidecar format that carries a font's advance
widths, bounding box, and per-glyph metrics — still produced by some
type tools and still consumed by PDF and PostScript workflows. It's
plain text, which means it's easy to hand-edit, which means it's easy
to quietly break: a stray glyph left out of the declared count, two
glyphs with the same name, a bounding box with the corners swapped.
Nothing refuses to load a broken AFM file until something downstream
mis-renders text and someone has to guess why.

`afm-lint` reads a file and reports problems with a line number, the
same way a compiler warning would.

## Usage

```
afm-lint path/to/font.afm
```

Pass several files, a directory, or a mix of both. Directories are walked
recursively and every `.afm` file found is linted; each file's findings are
reported the same way as a single-file run, one file after another:

```
afm-lint fonts/regular.afm fonts/bold.afm
afm-lint fonts/
```

Given a file where the character count in `StartCharMetrics` doesn't
match the glyphs actually listed, and a duplicate glyph name:

```
$ afm-lint broken.afm
broken.afm:9: error: StartCharMetrics declares 3 glyphs but 2 were found
broken.afm:12: error: glyph 'A' is already defined at line 11
```

The process exits 1 if any file produced an error-level finding, 0
otherwise (0 also covers "no issues found"). It exits 2 if a given path
couldn't be read or no `.afm` files were found at all.

### Strict by default, `--lenient` to relax it

By default `afm-lint` treats a missing recommended header field (like
`Version` or `Ascender`) or a malformed bounding box as an error — the
same severity as structural corruption. That's deliberate: this tool
is meant to run in CI against font metrics you author yourself, where
"technically loads" isn't the bar.

If you're pointing it at a third-party or legacy AFM file that you
can't fix and just want to catch actual corruption, pass `--lenient`.
It downgrades those style/completeness checks to warnings (or drops
them) and only fails on things that make the file structurally
unusable — a missing `EndCharMetrics`, a glyph line with no width, a
duplicate name:

```
afm-lint --lenient path/to/font.afm
```

## What it checks today

- `StartFontMetrics` / `EndFontMetrics` present
- `FontName` present
- recommended header keys present (`FontBBox`, `UnderlinePosition`,
  `UnderlineThickness`, `Version`, `Ascender`, `Descender`)
- `StartCharMetrics` is closed by a matching `EndCharMetrics`
- the glyph count declared on `StartCharMetrics` matches the number of
  glyph lines actually present
- `FontBBox` has positive area
- every glyph line has a name (`N`) and a width (`WX`)
- no two glyphs share a name
- no glyph has a negative width; a zero width on anything but `space`
  is flagged too

## Building

Standard library only, no external crates:

```
cargo build --release
```

## Testing

`tests/golden.rs` runs the built binary against fixture `.afm` files under
`tests/fixtures/` and checks stdout against a checked-in expected output,
covering both strict and `--lenient` runs:

```
cargo test
```

When a rule's message text changes, update the matching `.stdout` file
alongside it.

## License

MIT, see `LICENSE`.
