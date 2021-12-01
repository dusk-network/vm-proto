use microkelvin::{Ident, Offset, Storage, Store};
use rkyv::{ser::Serializer, Fallible};

#[cfg(not(feature = "host"))]
mod ext {
    #[link(wasm_import_module = "env")]
    extern "C" {
        pub fn debug(ofs: &u8, len: i32);
    }
}

#[cfg(not(feature = "host"))]
pub fn debug(string: &'static str) {
    let bytes = string.as_bytes();
    unsafe { ext::debug(&bytes[0], bytes.len() as i32) }
}

// Host mockups of the ABI
#[cfg(feature = "host")]
pub fn debug(string: &'static str) {
    println!("HOST DEBUG: {}", string)
}

/// Store backend over FFI

#[derive(Clone)]
pub struct AbiStore;

#[derive(Clone)]
pub struct AbiStorage;

impl Serializer for AbiStorage {
    fn pos(&self) -> usize {
        todo!()
    }

    fn write(&mut self, bytes: &[u8]) -> Result<(), Self::Error> {
        todo!()
    }
}

impl Fallible for AbiStore {
    type Error = core::convert::Infallible;
}

impl Fallible for AbiStorage {
    type Error = core::convert::Infallible;
}

impl Storage<Offset> for AbiStorage {
    fn put<T>(&mut self, t: &T) -> Offset
    where
        T: rkyv::Serialize<Self>,
    {
        todo!()
    }

    fn get<T>(&self, id: &Offset) -> &T::Archived
    where
        T: rkyv::Archive,
    {
        todo!()
    }
}

impl Store for AbiStore {
    type Identifier = Offset;

    type Storage = AbiStorage;

    fn put<T>(&self, t: &T) -> microkelvin::Stored<T, Self>
    where
        T: rkyv::Serialize<Self::Storage>,
    {
        todo!()
    }

    fn get_raw<'a, T>(&'a self, ident: &Ident<Self::Identifier, T>) -> &'a T::Archived
    where
        T: rkyv::Archive,
    {
        todo!()
    }
}
