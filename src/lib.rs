#![forbid(unsafe_code)]
#![deny(missing_debug_implementations)]
#![warn(clippy::all)]

use std::iter::FusedIterator;
use std::ops::Range;

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum EveryRangeKind {
    Included,
    Excluded,
}

// TODO: EveryRangeIter is not very lenient, consider if `range.start > self.end` and `range.end > self.end` should stop the iterator, instead of panicking
// TODO: The question is, if so, does it ignore the last range? does it clamp it? does it just return it anyways and stop after?

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

pub trait EveryRange: Sized + Iterator<Item = Range<usize>> {
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
