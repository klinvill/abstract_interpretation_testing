extern crate stable_mir as smir;

use crate::domains::domain::AbstractDomain;
use std::cmp::Ordering;
use smir::ty::{TyKind, RigidTy};

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum AbstractBool {
    Top, // Both True and False
    True,
    False,
    Bot, // Neither True nor False
}

impl PartialOrd for AbstractBool {
    /// AbstractBool is the lattice:
    ///          Top
    ///       /      \
    ///    False    True
    ///       \     /
    ///         Bot
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (AbstractBool::Top, AbstractBool::Top) => Some(Ordering::Equal),
            (AbstractBool::Top, _) => Some(Ordering::Greater),
            (_, AbstractBool::Top) => Some(Ordering::Less),
            (AbstractBool::Bot, AbstractBool::Bot) => Some(Ordering::Equal),
            (AbstractBool::Bot, _) => Some(Ordering::Less),
            (_, AbstractBool::Bot) => Some(Ordering::Greater),
            (a, b) => {
                if a == b {
                    Some(Ordering::Equal)
                } else {
                    None
                }
            }
        }
    }
}

impl AbstractDomain for AbstractBool {
    // fn abstraction(concrete: bool) -> Self {
    //     match concrete {
    //         false => AbstractBool::False,
    //         true => AbstractBool::True,
    //     }
    // }

    fn join(&self, other: &Self) -> Self {
        match self.partial_cmp(other) {
            None => AbstractBool::Top, // Only can't compare True and False, so Top is the join
            Some(Ordering::Equal) => *self,
            Some(Ordering::Less) => *other,
            Some(Ordering::Greater) => *self,
        }
    }

    fn widen(&self, other: &Self) -> Self {
        match (self, other) {
            (AbstractBool::Top, _) => AbstractBool::Top,
            (_, AbstractBool::Top) => AbstractBool::Top,
            (AbstractBool::Bot, _) => *other,
            (_, AbstractBool::Bot) => *self,
            (a, b) => {
                if a == b {
                    *a
                } else {
                    AbstractBool::Top
                }
            }
        }
    }

    fn top(&self) -> Self {
        AbstractBool::Top
    }
}

impl From<bool> for AbstractBool {
    fn from(concrete: bool) -> Self {
        match concrete {
            false => AbstractBool::False,
            true => AbstractBool::True,
        }
    }
}

impl From<&smir::ty::Const> for AbstractBool {
    fn from(constant: &smir::ty::Const) -> Self {
        match &constant.ty().kind()  {
            TyKind::RigidTy(RigidTy::Bool) => (),
            _ => panic!("Cannot construct an abstract boolean from a non-boolean constant."),
        };

        match &constant.kind() {
            smir::ty::ConstantKind::Allocated(alloc) => match alloc.bytes[..] {
                [Some(0)] => AbstractBool::False,
                [Some(_)] => AbstractBool::True,
                _ => panic!("Unexpected bytes when trying to convert from boolean constant to abstract boolean")
            }

            _ => panic!("From not implemented yet for boolean constants with a ConstantKind other than Allocated"),
        }
    }
}

impl AbstractBool {
    /// Abstract boolean equality operation
    pub(crate) fn equals(&self, other: &Self) -> AbstractBool {
        match (self, other) {
            (AbstractBool::Top, _) | (_, AbstractBool::Top) => AbstractBool::Top,
            (AbstractBool::Bot, _) | (_, AbstractBool::Bot) => AbstractBool::Bot,
            (a, b) => {
                if a == b {
                    AbstractBool::True
                } else {
                    AbstractBool::False
                }
            }
        }
    }
}
