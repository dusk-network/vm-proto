use microkelvin::{Ident, Offset, Storage, Store, Stored};
use rkyv::{ser::Serializer, Fallible};

pub struct AbiStorage;

#[derive(Clone)]
pub struct AbiStore;

impl Fallible for AbiStore {
    type Error = core::convert::Infallible;
}

impl Fallible for AbiStorage {
    type Error = core::convert::Infallible;
}

impl Serializer for AbiStorage {
    fn pos(&self) -> usize {
        todo!()
    }

    fn write(&mut self, _: &[u8]) -> Result<(), <Self as Fallible>::Error> {
        todo!()
    }
}

impl Storage<Offset> for AbiStorage {
    fn put<T>(&mut self, _t: &T) -> Offset
    where
        T: rkyv::Serialize<Self>,
    {
        todo!()
    }

    fn get<T>(&self, _id: &Offset) -> &T::Archived
    where
        T: rkyv::Archive,
    {
        todo!()
    }
}

impl Store for AbiStore {
    type Identifier = Offset;

    type Storage = AbiStorage;

    fn put<T>(&self, _t: &T) -> Stored<T, Self>
    where
        T: rkyv::Serialize<Self::Storage>,
    {
        todo!()
    }

    fn get_raw<'a, T>(&'a self, _ident: &Ident<Self::Identifier, T>) -> &'a T::Archived
    where
        T: rkyv::Archive,
    {
        todo!()
    }
}
