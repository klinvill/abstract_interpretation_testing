use crate::domains::value::AbstractValue;

/// Abstraction of a function as input and output abstract elements
#[derive(Debug)]
pub struct AbstractFunction {
    pub arguments: Vec<AbstractValue>,
    pub return_val: AbstractValue,
}
