use crate::domains::domain::AbstractDomain;
use std::cmp::Ordering;
use crate::domains::booleans::AbstractBool;

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
    pub(crate) lower: IntervalElem<T>,
    pub(crate) upper: IntervalElem<T>,
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

impl<T: Ord + Copy> Interval<T> {
    pub(crate) fn from_interval(concrete_lower: T, concrete_upper: T) -> Self {
        Interval {
            lower: IntervalElem::Elem(concrete_lower),
            upper: IntervalElem::Elem(concrete_upper),
        }
    }

    /// Abstract boolean equality operation
    pub(crate) fn equals(&self, other: &Self) -> AbstractBool {
        if self.upper < other.lower || other.upper < self.lower {
            // The two intervals don't overlap, always will be false
            AbstractBool::False
        } else if self.lower == self.upper && other.lower == other.upper && self.lower == other.lower && self.lower != IntervalElem::Inf && self.lower != IntervalElem::NegInf {
            // The two intervals consist of the same exact element (that's neither top nor bottom), will always be true
            AbstractBool::True
        } else {
            // There is at least some overlap in the intervals so it could be true or false
            AbstractBool::Top
        }
    }

    /// Abstract boolean less than operation
    pub(crate) fn less_than(&self, other: &Self) -> AbstractBool {
        if self.upper < other.lower {
            // The self interval is outside of and always less than the other interval
            AbstractBool::True
        } else if self.lower >= other.upper {
            // Elements in the self interval will always be greater than or equal to elements in the other interval
            AbstractBool::False
        } else {
            // There is at least some overlap in the intervals so it could be true or false
            AbstractBool::Top
        }
    }
}

impl <T: Ord + Copy + std::ops::Add<Output = T>> std::ops::Add for IntervalElem<T>
{
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        match (self, rhs) {
            (IntervalElem::NegInf, _) | (_, IntervalElem::NegInf) => IntervalElem::NegInf,
            (IntervalElem::Inf, _) | (_, IntervalElem::Inf) => IntervalElem::Inf,
            (IntervalElem::Elem(l), IntervalElem::Elem(r)) => IntervalElem::Elem(l + r),
        }
    }
}

impl <T: Ord + Copy + std::ops::Add> std::ops::Add for Interval<T>
    where IntervalElem<T>: std::ops::Add<Output = IntervalElem<T>>
{
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self {
            lower: self.lower + rhs.lower,
            upper: self.upper + rhs.upper,
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_less_than() {
        assert_eq!(
            Interval::from(10).less_than(
                &Interval::from(10)
            ),
            // Equality always means not less than
            AbstractBool::False
        );
        assert_eq!(
            Interval::from_interval(-10, 5).less_than(
                &Interval::from_interval(20, 30)
            ),
            // Disjoint intervals where the first is less than the second always means less than
            AbstractBool::True
        );
        assert_eq!(
            Interval::from_interval(20, 30).less_than(
                &Interval::from_interval(-10, 5)
            ),
            // Disjoint intervals where the first is greater than the second always means not less than
            AbstractBool::False
        );
        assert_eq!(
            Interval::from_interval(20, 30).less_than(
                &Interval::from_interval(0, 20)
            ),
            // Intervals where the first is greater than or equal to the second always means not less than
            AbstractBool::False
        );
        assert_eq!(
            Interval::from_interval(10, 30).less_than(
                &Interval::from_interval(0, 15)
            ),
            // Overlapping intervals could be either less than or not less than
            AbstractBool::Top
        );
        assert_eq!(
            Interval::from_interval(10, 20).less_than(
                &Interval::from_interval(20, 30)
            ),
            // If the upper bound of the first is equal to the lower bound of the second, it could be either less than or not less than
            AbstractBool::Top
        );
        assert_eq!(
            Interval { lower: IntervalElem::NegInf, upper: IntervalElem::Elem(5) }.less_than(
                &Interval { lower: IntervalElem::Elem(20), upper: IntervalElem::Inf }
            ),
            // Disjoint intervals where the first is less than the second always means less than even when an infinite bound is included
            AbstractBool::True
        );
        assert_eq!(
            Interval::from_interval(10, 30).less_than(
                &Interval::from(0).top()
            ),
            // Top overlaps with everything so it could be either less than or not less than
            AbstractBool::Top
        );
        assert_eq!(
            Interval::from(0).top().less_than(
                &Interval::from_interval(0, 15)
            ),
            // Top overlaps with everything so it could be either less than or not less than
            AbstractBool::Top
        );
    }
}
