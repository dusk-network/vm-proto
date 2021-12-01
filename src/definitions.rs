use core::fmt::Debug;

use microkelvin::Store;
use rkyv::Archive;

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug, Default)]
pub struct ContractId([u8; 32]);

pub trait Execute<Q, S>
where
    Self: Archive,
    S: Store,
    Q: Query,
{
    fn execute(archived: &Self::Archived, q: &Q::Archived, store: &S) -> Q::Return;
}

pub trait Apply<T, S>
where
    T: Transaction,
    S: Store,
{
    fn apply(&mut self, t: &T::Archived, store: &S) -> T::Return;
}

pub trait Query: Archive {
    const NAME: &'static str;

    type Return;
}

pub trait Transaction: Archive {
    const NAME: &'static str;

    type Return;
}
