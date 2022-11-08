// Interfaces
mod domain;
mod function;
mod value;

// Domains
pub(crate) mod booleans;
pub(crate) mod interval;

pub use function::AbstractFunction;
pub use value::AbstractValue;
