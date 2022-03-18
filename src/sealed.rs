#[allow(dead_code)]
#[allow(path_statements)]
pub(crate) const fn greater_than_eq<const N: usize, const MIN: usize>() {
    #[allow(clippy::no_effect)]
    Assert::<N, MIN>::GREATER_EQ;
}

/// Const assert hack
#[allow(dead_code)]
pub struct Assert<const L: usize, const R: usize>;

#[allow(dead_code)]
impl<const L: usize, const R: usize> Assert<L, R> {
    /// Const assert hack
    pub const GREATER_EQ: usize = L - R;
}
