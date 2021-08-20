use core::{fmt::Debug, pin::Pin};

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug, Default)]
pub struct ContractId([u8; 32]);

pub trait Method {
    const NAME: &'static str;
    type Return;
}

pub trait Query<Q: Method> {
    fn query(&self, q: &Q) -> Q::Return;
}

pub trait Apply<T: Method> {
    fn apply(self: Pin<&mut Self>, t: &T) -> T::Return;
}
