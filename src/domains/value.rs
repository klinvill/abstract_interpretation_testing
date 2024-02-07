extern crate rustc_middle;
extern crate stable_mir as smir;

use crate::errors::{Error, ErrorKind};

use crate::domains::booleans;
use crate::domains::domain::AbstractDomain;
use crate::domains::interval;

use smir::ty::{TyKind, RigidTy};

#[derive(Debug, Clone, PartialEq)]
pub enum AbstractValue {
    Bool(booleans::AbstractBool),
    // TODO(klinvill): A function that returns different kinds of AbstractValue types based on the
    //  input (e.g. the abstract_value_from_type() function) can't support AbstractValue having a
    //  generic parameter. Instead, we specialize a few intervals here to handle signed ints and
    //  unsigned ints. We use the largest primitive size for each category. Could we do this better?
    //
    // Note: float values in rust (e.g. f64) do not implement Ord, only PartialOrd, so they can't
    //  be intervals.
    IntInterval(interval::Interval<i128>),
    UintInterval(interval::Interval<u128>),
    Tuple(Vec<AbstractValue>),
    // Value that represents an unitialized value.Can be explicitly created through a statement like Deinit.
    Uninit,
}

impl AbstractDomain for AbstractValue {
    fn join(&self, other: &Self) -> Self {
        match (self, other) {
            (AbstractValue::Bool(a), AbstractValue::Bool(b)) => AbstractValue::Bool(a.join(b)),
            // TODO(klinvill): handle conversions between int, uint, etc.
            (AbstractValue::IntInterval(a), AbstractValue::IntInterval(b)) => AbstractValue::IntInterval(a.join(b)),
            (AbstractValue::UintInterval(a), AbstractValue::UintInterval(b)) => AbstractValue::UintInterval(a.join(b)),
            (_, _) => panic!("Can only perform operations on abstract values of the same type (e.g. Bool or IntInterval)"),
        }
    }

    fn widen(&self, other: &Self) -> Self {
        match (self, other) {
            (AbstractValue::Bool(a), AbstractValue::Bool(b)) => AbstractValue::Bool(a.join(b)),
            // TODO(klinvill): handle conversions between int, uint, etc.
            (AbstractValue::IntInterval(a), AbstractValue::IntInterval(b)) => AbstractValue::IntInterval(a.widen(b)),
            (AbstractValue::UintInterval(a), AbstractValue::UintInterval(b)) => AbstractValue::UintInterval(a.widen(b)),
            (_, _) => panic!("Can only perform operations on abstract values of the same type (e.g. Bool or IntInterval)"),
        }
    }

    fn top(&self) -> Self {
        match self {
            AbstractValue::Bool(x) => AbstractValue::Bool(x.top()),
            AbstractValue::IntInterval(x) => AbstractValue::IntInterval(x.top()),
            AbstractValue::UintInterval(x) => AbstractValue::UintInterval(x.top()),
            AbstractValue::Tuple(avs) => {
                AbstractValue::Tuple(avs.iter().map(|x| x.top()).collect())
            }
            AbstractValue::Uninit => AbstractValue::Uninit,
        }
    }
}

impl AbstractValue {
    // TODO(klinvill): Should new return a result or always return a successful object? Should this
    //  be renamed to try_new()?
    pub fn new(ty: &smir::ty::Ty) -> Result<Self, Error> {
        match ty.kind() {
            TyKind::RigidTy(RigidTy::Bool) => Ok(AbstractValue::Bool(booleans::AbstractBool::Top)),
            TyKind::RigidTy(RigidTy::Int(_)) => Ok(AbstractValue::IntInterval(
                interval::Interval::from(0).top(),
            )),
            TyKind::RigidTy(RigidTy::Uint(_)) => Ok(AbstractValue::UintInterval(
                interval::Interval::from(0).top(),
            )),
            TyKind::RigidTy(RigidTy::Tuple(tys)) => {
                let try_avs: Result<Vec<AbstractValue>, _> =
                    tys.iter().map(|t| AbstractValue::new(&t)).collect();
                try_avs.map(AbstractValue::Tuple)
            }
            _ => Err(Error::new(ErrorKind::NotImplementedError)),
        }
    }

    pub fn get(&self, index: usize) -> Option<&Self> {
        match self {
            AbstractValue::Tuple(entries) => entries.get(index),
            _ => None,
        }
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut Self> {
        match self {
            AbstractValue::Tuple(entries) => entries.get_mut(index),
            _ => None,
        }
    }

    pub fn set(&mut self, index: usize, value: AbstractValue) -> Result<(), Error> {
        match self {
            AbstractValue::Tuple(entries) => {
                if index < entries.len() {
                    entries[index] = value;
                    Ok(())
                } else {
                    Err(Error::with_message(
                        ErrorKind::InterpreterError,
                        "Tried to index entry outside tuple limits".to_string()
                    ))
                }
            },
            _ => Err(Error::with_message(
                ErrorKind::NotImplementedError,
                format!("set not implemented for abstract value: {:?}", value).to_string()
            )),
        }
    }
}
