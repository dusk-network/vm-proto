use core::fmt::Debug;

use rkyv::Archive;

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug, Default)]
pub struct ContractId([u8; 32]);

pub trait Method {
    const NAME: &'static str;
    type Return;
}

pub trait Query<Q>
where
    Self: Archive,
    Q: Method + Archive,
{
    fn query(archived: &Self::Archived, q: &Q::Archived) -> Q::Return;
}

pub trait Apply<T>
where
    T: Method,
{
    fn apply(&mut self, t: T) -> T::Return;
}
