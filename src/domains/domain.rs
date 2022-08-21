pub trait AbstractDomain {
    // Note: Ideally we could require an abstraction function with a generic type
    // argument which would mean we can't use AbstractDomain as a trait object (but it already
    // can't be a trait object unless join is changed to take an argument with a type other than
    // Self). That in turn prevents writing a function that can return AbstractBools that can only
    // abstract concrete bool types, and also return Intervals that can abstract any ordered (e.g.
    // numeric) type.
    // fn abstraction(concrete: T) -> Self;

    fn join(&self, other: &Self) -> Self;
    fn widen(&self, other: &Self) -> Self;
    /// Get the top element in the lattice. For booleans, this is Top. For intervals, this is [min,max].
    fn top(&self) -> Self;
}
