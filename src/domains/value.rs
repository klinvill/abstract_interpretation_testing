extern crate rustc_middle;

use crate::errors::{Error, ErrorKind};

use crate::domains::booleans;
use crate::domains::domain::AbstractDomain;
use crate::domains::interval;

#[derive(Debug)]
pub enum AbstractValue {
    Bool(booleans::AbstractBool),
    // TODO(klinvill): A function that returns different kinds of AbstractValue types based on the
    //  input (e.g. the abstract_value_from_type() function) can't support AbstractValue having a
    //  generic parameter. Instead, we specialize a few intervals here to handle signed ints and
    //  unsigned ints. We use the largest primitive size for each category. Could we do this better?
    // Note: float values in rust (e.g. f64) do not implement Ord, only PartialOrd, so they can't
    //  be intervals.
    IntInterval(interval::Interval<i128>),
    UintInterval(interval::Interval<u128>),
    Tuple(Vec<AbstractValue>),
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
        }
    }
}

impl AbstractValue {
    // TODO(klinvill): Should new return a result or always return a successful object? Should this
    //  be renamed to try_new()?
    // TODO(klinvill): Should replace ty type with &smir::ty::Ty, but it looks like the latest
    //  change including ty in smir hasn't been synced back to rust yet.
    pub fn new(ty: &rustc_middle::ty::Ty) -> Result<Self, Error> {
        match ty.kind() {
            // TODO(klinvill): Should replace with smir::ty::TyKind types, but it looks like the
            //  latest change including ty in smir hasn't been synced back to rust yet.
            rustc_middle::ty::TyKind::Bool => Ok(AbstractValue::Bool(booleans::AbstractBool::Top)),
            rustc_middle::ty::TyKind::Int(_) => Ok(AbstractValue::IntInterval(
                interval::Interval::from(0).top(),
            )),
            rustc_middle::ty::TyKind::Uint(_) => Ok(AbstractValue::UintInterval(
                interval::Interval::from(0).top(),
            )),
            rustc_middle::ty::TyKind::Tuple(tys) => {
                let try_avs: Result<Vec<AbstractValue>, _> =
                    tys.iter().map(|t| AbstractValue::new(&t)).collect();
                try_avs.map(AbstractValue::Tuple)
            }
            _ => Err(Error::new(ErrorKind::NotImplementedError)),
        }
    }
}
