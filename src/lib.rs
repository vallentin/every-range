//! This crate implements an extension to [`Iterator`],
//! which features an [`every_range`] method
//! on any [`Iterator`] with an [`Item`] of [`Range<usize>`].
//!
//! [`every_range`]: trait.EveryRange.html#method.every_range
//!
//! [`Iterator`]: https://doc.rust-lang.org/stable/std/iter/trait.Iterator.html
//! [`Item`]: https://doc.rust-lang.org/stable/std/iter/trait.Iterator.html#associatedtype.Item
//!
//! [`Range`]: https://doc.rust-lang.org/stable/std/ops/struct.Range.html
//! [`Range<usize>`]: https://doc.rust-lang.org/stable/std/ops/struct.Range.html
//!
//! [`EveryRangeIter`] iterates over [`Range`]s and "fill in"
//! missing ranges, i.e. the gap between two consecutive ranges.
//! The original ranges and the generated ones,
//! can be distinguished by the [`Included`] and
//! [`Excluded`] enum variants.
//!
//! [`EveryRangeIter`]: struct.EveryRangeIter.html
//! [`Included`]: enum.EveryRangeKind.html#variant.Included
//! [`Excluded`]: enum.EveryRangeKind.html#variant.Excluded
//!
//! [`EveryRangeIter`] is useful when the ranges being iterated
//! are related to substrings that later are used to replaced
//! parts in a string.
//!
//! # Example: How does it work?
//!
//! ```no_run
//! use every_range::EveryRange;
//!
//! // Lets use text as an example, but it could be anything
//! let text = "Foo rust-lang.org Bar
//! Baz crates.io Qux";
//!
//! // Get some ranges from somewhere
//! let ranges = vec![
//!     4..17,  // "rust-lang.org"
//!     26..35, // "crates.io"
//! ];
//!
//! // `text.len()` tells `EveryRange` the end, so it knows
//! // whether to produce an extra range after or not
//! let iter = ranges.into_iter().every_range(text.len());
//!
//! // The input `ranges` result in `Included` every other range is `Excluded`
//! for (kind, range) in iter {
//!     println!("{:?} {:>2?} - {:?}", kind, range.clone(), &text[range]);
//! }
//! ```
//!
//! This will output the following:
//!
//! ```text
//! Excluded  0.. 4 - "Foo "
//! Included  4..17 - "rust-lang.org"
//! Excluded 17..26 - " Bar\nBaz "
//! Included 26..35 - "crates.io"
//! Excluded 35..39 - " Qux"
//! ```
//!
//! # Example: "Autolink" or HTMLify URLs
//!
//! Using [`every_range`] it is easy to collect ranges or
//! substring into a [`String`].
//!
//! [`String`]: https://doc.rust-lang.org/stable/std/string/struct.String.html
//!
//! ```no_run
//! use std::borrow::Cow;
//! use every_range::{EveryRange, EveryRangeKind};
//!
//! let text = "Foo rust-lang.org Bar
//! Baz crates.io Qux";
//!
//! // For URLs input ranges could be produced by linkify
//! let ranges = vec![
//!     4..17,  // "rust-lang.org"
//!     26..35, // "crates.io"
//! ];
//!
//! let output = ranges
//!     .into_iter()
//!     .every_range(text.len())
//!     .map(|(kind, range)| {
//!         if kind == EveryRangeKind::Included {
//!             let url = &text[range];
//!             format!("<a href=\"{0}\">{0}</a>", url).into()
//!         } else {
//!             Cow::Borrowed(&text[range])
//!         }
//!     })
//!     .collect::<Vec<_>>()
//!     .concat();
//!
//! println!("{}", output);
//! ```
//!
//! This will output the following:
//!
//! ```text
//! Foo <a href="rust-lang.org">rust-lang.org</a> Bar
//! Baz <a href="crates.io">crates.io</a> Qux
//! ```

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![warn(clippy::all)]

use std::iter::FusedIterator;
use std::ops::Range;

/// `EveryRangeKind` can be used to distinguish original input
/// ranges from generates ranges.
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum EveryRangeKind {
    /// `Included` ranges are the ones produces by the inner [`Iterator`].
    ///
    /// [`Iterator`]: https://doc.rust-lang.org/stable/std/iter/trait.Iterator.html
    Included,

    /// Excluded ranges are the ones generated dynamically by [`EveryRangeIter`].
    ///
    /// [`EveryRangeIter`]: struct.EveryRangeIter.html
    Excluded,
}

// TODO: EveryRangeIter is not very lenient, consider if `range.start > self.end` and `range.end > self.end` should stop the iterator, instead of panicking
// TODO: The question is, if so, does it ignore the last range? does it clamp it? does it just return it anyways and stop after?

/// `EveryRangeIter` iterates over [`Range`]s and "fill in"
/// missing ranges, i.e. the gap between two consecutive ranges.
/// The original ranges and the generated ones,
/// can be distinguished by the [`Included`] and
/// [`Excluded`] enum variants.
///
/// [`EveryRangeIter`]: struct.EveryRangeIter.html
/// [`Included`]: enum.EveryRangeKind.html#variant.Included
/// [`Excluded`]: enum.EveryRangeKind.html#variant.Excluded
///
/// [`Range`]: https://doc.rust-lang.org/stable/std/ops/struct.Range.html
///
/// # Panics
///
/// Currently, `EveryRangeIter` resorts to panicking
/// in the following conditions. `EveryRangeIter` might
/// be made more lenient in the future, if the behavior
/// can be better consistently defined without panicking.
///
/// - Panics if [`Range`]s are received out of order.
/// - Panics if [`Range`]s overlap.
/// - Panics if any [`Range`] exceeds the `end` of the `EveryRangeIter`.
#[allow(missing_debug_implementations)]
pub struct EveryRangeIter<I>
where
    I: Iterator<Item = Range<usize>>,
{
    index: usize,
    end: usize,
    iter: I,
    next: Option<Range<usize>>,
}

impl<I> EveryRangeIter<I>
where
    I: Iterator<Item = Range<usize>>,
{
    /// Create an `EveryRangeIter` with an `iter` and `end`,
    /// which represents the "end point". Thereby, if `end` is
    /// greater than the last [`range.end`] then an ending
    /// [`Excluded`] range is generated, otherwise no additional
    /// ending range is generated.
    ///
    /// [`Excluded`]: enum.EveryRangeKind.html#variant.Excluded
    ///
    /// [`range.end`]: https://doc.rust-lang.org/stable/std/ops/struct.Range.html#structfield.end
    #[inline]
    pub fn new(iter: I, end: usize) -> Self {
        Self {
            index: 0,
            end,
            iter,
            next: None,
        }
    }
}

impl<I> Iterator for EveryRangeIter<I>
where
    I: Iterator<Item = Range<usize>>,
{
    type Item = (EveryRangeKind, Range<usize>);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(next) = self.next.take() {
            self.index = next.end;

            Some((EveryRangeKind::Included, next))
        } else if let Some(next) = self.iter.next() {
            assert!(self.index <= next.start);
            assert!(next.end <= self.end);
            assert!(next.start <= next.end);

            if self.index < next.start {
                let start = self.index;
                self.index = next.start;
                self.next = Some(next);

                Some((EveryRangeKind::Excluded, start..self.index))
            } else {
                self.index = next.end;

                Some((EveryRangeKind::Included, next))
            }
        } else if self.index < self.end {
            let start = self.index;

            self.index = self.end;

            Some((EveryRangeKind::Excluded, start..self.end))
        } else {
            None
        }
    }
}

impl<I> FusedIterator for EveryRangeIter<I> where I: Iterator<Item = Range<usize>> {}

/// Trait which implements `every_range` to get a `EveryRangeIter`.
///
/// *[See `EveryRangeIter` for more information.][`EveryRangeIter`]*
///
/// [`every_range`]: trait.EveryRange.html#method.every_range
/// [`EveryRangeIter`]: struct.EveryRangeIter.html
pub trait EveryRange: Sized + Iterator<Item = Range<usize>> {
    /// Create an [`EveryRangeIter`] with `end`, which represents
    /// the "end point". Thereby, if `end` is greater than the last
    /// [`range.end`] then an ending [`Excluded`] range is generated,
    /// otherwise no additional ending range is generated.
    ///
    /// *[See `EveryRangeIter` for more information.][`EveryRangeIter`]*
    ///
    /// [`EveryRangeIter`]: struct.EveryRangeIter.html
    /// [`Excluded`]: enum.EveryRangeKind.html#variant.Excluded
    ///
    /// [`range.end`]: https://doc.rust-lang.org/stable/std/ops/struct.Range.html#structfield.end
    #[inline]
    fn every_range(self, end: usize) -> EveryRangeIter<Self> {
        EveryRangeIter::new(self, end)
    }
}

impl<T> EveryRange for T where T: Iterator<Item = Range<usize>> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_range_matches1() {
        let text = "Foo12Bar34Baz56";

        use EveryRangeKind::*;
        let expected = [
            ((Included, 0..1), "F"),
            ((Included, 1..2), "o"),
            ((Included, 2..3), "o"),
            ((Excluded, 3..5), "12"),
            ((Included, 5..6), "B"),
            ((Included, 6..7), "a"),
            ((Included, 7..8), "r"),
            ((Excluded, 8..10), "34"),
            ((Included, 10..11), "B"),
            ((Included, 11..12), "a"),
            ((Included, 12..13), "z"),
            ((Excluded, 13..15), "56"),
        ];

        let mut iter_actual = text
            .match_indices(char::is_alphabetic)
            .map(|(start, part)| {
                let end = start + part.len();
                start..end
            })
            .every_range(text.len())
            .map(|(kind, range)| ((kind, range.clone()), &text[range]));

        for expected in expected.iter().cloned() {
            assert_eq!(Some(expected), iter_actual.next());
        }

        assert_eq!(None, iter_actual.next());
    }

    #[test]
    fn every_range_matches2() {
        let text = "Foo12Bar34Baz56";

        use EveryRangeKind::*;
        let expected = [
            ((Excluded, 0..3), "Foo"),
            ((Included, 3..4), "1"),
            ((Included, 4..5), "2"),
            ((Excluded, 5..8), "Bar"),
            ((Included, 8..9), "3"),
            ((Included, 9..10), "4"),
            ((Excluded, 10..13), "Baz"),
            ((Included, 13..14), "5"),
            ((Included, 14..15), "6"),
        ];

        let mut iter_actual = text
            .match_indices(char::is_numeric)
            .map(|(start, part)| {
                let end = start + part.len();
                start..end
            })
            .every_range(text.len())
            .map(|(kind, range)| ((kind, range.clone()), &text[range]));

        for expected in expected.iter().cloned() {
            assert_eq!(Some(expected), iter_actual.next());
        }

        assert_eq!(None, iter_actual.next());
    }

    #[test]
    #[should_panic = "assertion failed: next.end <= self.end"]
    fn range_start_after_end() {
        [0..2, 4..6].iter().cloned().every_range(3).for_each(|_| {});
    }

    #[test]
    #[should_panic = "assertion failed: next.end <= self.end"]
    fn range_end_after_end() {
        [0..2, 4..6].iter().cloned().every_range(5).for_each(|_| {});
    }

    #[test]
    #[should_panic = "assertion failed: self.index <= next.start"]
    fn range_start_after_index() {
        [0..4, 2..6].iter().cloned().every_range(5).for_each(|_| {});
    }

    #[test]
    #[should_panic = "assertion failed: self.index <= next.start"]
    fn ranges_out_of_order1() {
        [4..6, 0..2, 8..10]
            .iter()
            .cloned()
            .every_range(20)
            .for_each(|_| {});
    }

    #[test]
    #[should_panic = "assertion failed: self.index <= next.start"]
    fn ranges_out_of_order2() {
        [8..10, 0..2, 4..6]
            .iter()
            .cloned()
            .every_range(20)
            .for_each(|_| {});
    }

    #[test]
    #[should_panic = "assertion failed: self.index <= next.start"]
    fn ranges_out_of_order3() {
        [0..2, 8..10, 4..6]
            .iter()
            .cloned()
            .every_range(20)
            .for_each(|_| {});
    }

    #[test]
    #[should_panic = "assertion failed: self.index <= next.start"]
    fn ranges_out_of_order4() {
        [4..6, 8..10, 0..2]
            .iter()
            .cloned()
            .every_range(20)
            .for_each(|_| {});
    }

    #[test]
    #[should_panic = "assertion failed: self.index <= next.start"]
    fn ranges_out_of_order5() {
        [8..10, 4..6, 0..2]
            .iter()
            .cloned()
            .every_range(20)
            .for_each(|_| {});
    }
}
