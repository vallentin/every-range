# every-range

[![Build Status](https://travis-ci.org/vallentin/every-range.svg?branch=master)](https://travis-ci.org/vallentin/every-range)
[![Latest Version](https://img.shields.io/crates/v/every-range.svg)](https://crates.io/crates/every-range)
[![Docs](https://docs.rs/every-range/badge.svg)](https://docs.rs/every-range)
[![License](https://img.shields.io/github/license/vallentin/every-range.svg)](https://github.com/vallentin/every-range)

This crate implements an extension to [`Iterator`],
which features an [`every_range`] method
on any [`Iterator`] with an [`Item`] of [`Range<usize>`].

[`every_range`]: https://docs.rs/every-range/*/every_range/trait.EveryRange.html#method.every_range

[`Iterator`]: https://doc.rust-lang.org/stable/std/iter/trait.Iterator.html
[`Item`]: https://doc.rust-lang.org/stable/std/iter/trait.Iterator.html#associatedtype.Item

[`Range`]: https://doc.rust-lang.org/stable/std/ops/struct.Range.html
[`Range<usize>`]: https://doc.rust-lang.org/stable/std/ops/struct.Range.html

[`EveryRangeIter`] iterates over [`Range`]s and "fill in"
missing ranges, i.e. the gap between two consecutive ranges.
The original ranges and the generated ones,
can be distinguished by the [`Included`] and
[`Excluded`] enum variants.

[`EveryRangeIter`]: https://docs.rs/every-range/*/every_range/struct.EveryRangeIter.html
[`Included`]: https://docs.rs/every-range/*/every_range/enum.EveryRangeKind.html#variant.Included
[`Excluded`]: https://docs.rs/every-range/*/every_range/enum.EveryRangeKind.html#variant.Excluded

[`EveryRangeIter`] is useful when the ranges being iterated
are related to substrings that later are used to replaced
parts in a string.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
every-range = "0.1"
```

## Releases

Release notes are available in the repo at [CHANGELOG.md].

[CHANGELOG.md]: CHANGELOG.md

## Example: How does it work?

```rust
use every_range::EveryRange;

// Lets use text as an example, but it could be anything
let text = "Foo rust-lang.org Bar
Baz crates.io Qux";

// Get some ranges from somewhere
let ranges = vec![
    4..17,  // "rust-lang.org"
    26..35, // "crates.io"
];

// `text.len()` tells `EveryRange` the end, so it knows
// whether to produce an extra range after or not
let iter = ranges.into_iter().every_range(text.len());

// The input `ranges` result in `Included` every other range is `Excluded`
for (kind, range) in iter {
    println!("{:?} {:>2?} - {:?}", kind, range.clone(), &text[range]);
}
```

This will output the following:

```text
Excluded  0.. 4 - "Foo "
Included  4..17 - "rust-lang.org"
Excluded 17..26 - " Bar\nBaz "
Included 26..35 - "crates.io"
Excluded 35..39 - " Qux"
```

## Example: "Autolink" or HTMLify URLs

Using [`every_range`] it is easy to collect ranges or
substring into a [`String`].

[`String`]: https://doc.rust-lang.org/stable/std/string/struct.String.html

```rust
use std::borrow::Cow;
use every_range::{EveryRange, EveryRangeKind};

let text = "Foo rust-lang.org Bar
Baz crates.io Qux";

// For URLs input ranges could be produced by linkify
let ranges = vec![
    4..17,  // "rust-lang.org"
    26..35, // "crates.io"
];

let output = ranges
    .into_iter()
    .every_range(text.len())
    .map(|(kind, range)| {
        if kind == EveryRangeKind::Included {
            let url = &text[range];
            format!("<a href=\"{0}\">{0}</a>", url).into()
        } else {
            Cow::Borrowed(&text[range])
        }
    })
    .collect::<Vec<_>>()
    .concat();

println!("{}", output);
```

This will output the following:

```html
Foo <a href="rust-lang.org">rust-lang.org</a> Bar
Baz <a href="crates.io">crates.io</a> Qux
```
