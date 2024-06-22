pub trait SimplifyExt<F, P> {
    fn simplify(self) -> F;
}

impl<F, C, T, A0> SimplifyExt<F, A0> for C
where
    C: FnOnce(A0) -> T,
    F: FnOnce(A0),
{
    fn simplify(self) -> F {
        todo!()
    }
}
