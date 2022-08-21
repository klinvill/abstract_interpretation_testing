use crate::domains::domain::AbstractDomain;
use std::cmp::Ordering;

/// Represents an element in an interval, or +/- infinity for a bound of the interval.
#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum IntervalElem<T: Ord + Copy> {
    Inf,
    Elem(T),
    NegInf,
}

/// Represents an interval ranging between lower and upper.
#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct Interval<T: Ord + Copy> {
    lower: IntervalElem<T>,
    upper: IntervalElem<T>,
}

impl<T: Ord + Copy> PartialOrd for IntervalElem<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: Ord + Copy> Ord for IntervalElem<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (IntervalElem::Inf, IntervalElem::Inf) => Ordering::Equal,
            (IntervalElem::Inf, _) => Ordering::Greater,
            (_, IntervalElem::Inf) => Ordering::Less,
            (IntervalElem::NegInf, IntervalElem::NegInf) => Ordering::Equal,
            (IntervalElem::NegInf, _) => Ordering::Less,
            (_, IntervalElem::NegInf) => Ordering::Greater,
            // We require the elements themselves to be ordered so we can just re-use cmp().
            (IntervalElem::Elem(a), IntervalElem::Elem(b)) => a.cmp(b),
        }
    }
}

impl<T: Ord + Copy> AbstractDomain for Interval<T> {
    // fn abstraction(concrete: T) -> Self {
    //     Interval {
    //         lower: IntervalElem::Elem(concrete),
    //         upper: IntervalElem::Elem(concrete),
    //     }
    // }

    /// Merges two intervals such that the resulting interval contains both intervals
    fn join(&self, other: &Self) -> Self {
        Interval {
            lower: Ord::min(self.lower, other.lower),
            upper: Ord::max(self.upper, other.upper),
        }
    }

    fn widen(&self, other: &Self) -> Self {
        Interval {
            lower: if other.lower < self.lower {
                IntervalElem::NegInf
            } else {
                self.lower
            },
            upper: if other.upper > self.upper {
                IntervalElem::Inf
            } else {
                self.upper
            },
        }
    }

    fn top(&self) -> Self {
        Interval {
            lower: IntervalElem::NegInf,
            upper: IntervalElem::Inf,
        }
    }
}

impl<T: Ord + Copy> From<T> for Interval<T> {
    fn from(concrete: T) -> Self {
        Interval {
            lower: IntervalElem::Elem(concrete),
            upper: IntervalElem::Elem(concrete),
        }
    }
}
