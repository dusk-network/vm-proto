use std::fmt::Debug;

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug, Default)]
pub struct ContractId([u8; 32]);

pub trait Contract {
    fn code() -> &'static [u8];
}

pub trait Method {
    const NAME: &'static str;
    type Return;
}

pub trait Query<Q: Method> {
    fn query(&self, q: &Q) -> Q::Return;
}

pub trait Apply<T: Method> {
    fn apply(&mut self, t: &T) -> T::Return;
}
