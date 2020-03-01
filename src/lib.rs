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
