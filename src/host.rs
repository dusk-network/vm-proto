use std::collections::HashMap as Map;
use std::fmt::{Debug, Display};

use crate::definitions::*;

use rkyv::de::deserializers::*;
use rkyv::{ser::serializers::*, ser::Serializer, Archive};
use rkyv::{AlignedVec, Deserialize, Fallible, Infallible, Serialize};

use thiserror::Error;
use wasmer::{
    imports, Array, CompileError, ExportError, Instance, Module, RuntimeError, Store, Value,
    WasmPtr,
};

type DefaultSerializer = CompositeSerializer<
    AlignedSerializer<AlignedVec>,
    // This is the kind of scratch space we want to provide.
    // The 4096 means that we'll allocate 4KB of scratch space upfront and
    // then spill to dynamic allocations if we go over it.
    FallbackScratch<HeapScratch<4096>, AllocScratch>,
    // This is the shared serialize registry we want to provide.
    // Infallible in any of these arguments means we don't want to provide
    // that capability, but you can use SharedSerializeMap instead if you
    // want to serialize shared pointers. Only pay for what you need!
    Infallible,
>;
type SerializeResult<T> = Result<T, <DefaultSerializer as Fallible>::Error>;

// // We'll bound T by making sure that our default serializer can serialize it.
// // If we try to serialize a type that's not supported like a shared pointer,
// // it will be a compiler error until we add that capability to our serializer.
// // This function will return a result with the number of bytes written.
// fn serialize_into<'a, T>(value: &T, bytes: &'a mut [u8]) -> SerializeResult<'a, usize>
// where
//     T: Serialize<DefaultSerializer<'a>>,
// {
//     let mut serializer = DefaultSerializer::new(
//         // Our serializer is first, we need to pass it our bytes
//         WriteSerializer::new(bytes),
//         // The rest all implement Default
//         Default::default(),
//         Default::default(),
//     );
//     serializer.serialize_value(value)?;
//     Ok(serializer.pos())
// }

#[derive(Error, Debug)]
pub enum VMError {
    #[error("Unknown contract")]
    UnknownContract,
    #[error("{0}")]
    Exports(#[from] ExportError),
    #[error("{0}")]
    CompileError(#[from] CompileError),
    #[error("{0}")]
    RuntimeError(#[from] RuntimeError),
    #[error("{0}")]
    Other(String),
}

impl<A, B, C> From<CompositeSerializerError<A, B, C>> for VMError
where
    A: Display,
    B: Display,
    C: Display,
{
    fn from(comp: CompositeSerializerError<A, B, C>) -> Self {
        VMError::Other(format!("{}", comp))
    }
}

#[derive(Debug)]
struct ContractInstance {
    pub code: Vec<u8>,
    pub state: AlignedVec,
    pub state_ofs: i32,
}

impl ContractInstance {
    fn id(&self) -> ContractId {
        Default::default()
    }
}

#[derive(Debug, Default)]
pub struct State {
    map: Map<ContractId, ContractInstance>,
    wasmer_store: Store,
}

impl State {
    pub fn deploy<C>(&mut self, contract: C) -> Result<ContractId, VMError>
    where
        C: Contract + Debug + Serialize<DefaultSerializer>,
    {
        let mut serialize = DefaultSerializer::default();
        let state_ofs = serialize.serialize_value(&contract)?;
        let state = serialize.into_serializer().into_inner();

        let instance = ContractInstance {
            code: C::code().into(),
            state,
            state_ofs: state_ofs as i32,
        };

        let id = instance.id();

        self.map.insert(id, instance);
        Ok(id)
    }

    pub fn query<M>(&self, id: ContractId, arg: &M) -> Result<M::Return, VMError>
    where
        M: Method + Serialize<DefaultSerializer>,
    {
        if let Some(contract) = self.map.get(&id) {
            let mut serialize = DefaultSerializer::default();
            let argmt_ofs = serialize.serialize_value(arg)? as i32;
            let argmt = serialize.into_serializer().into_inner();

            let module = Module::new(&self.wasmer_store, &contract.code)?;
            let import_object = imports! {};
            let instance = Instance::new(&module, &import_object).unwrap();
            let function = instance
                .exports
                .get_native_function::<(i32, i32), WasmPtr<u8, Array>>(M::NAME)?;

            let call_result = function.call(contract.state_ofs, argmt_ofs)?;

            let memory = instance.exports.get_memory("memory")?;

            println!("mem: {:?}", memory);

            todo!()
        } else {
            Err(VMError::UnknownContract)
        }
    }

    pub fn apply<M>(&mut self, id: ContractId, method: &M) -> Result<M::Return, VMError>
    where
        M: Method,
    {
        if let Some(contract) = self.map.get(&id) {
            let module = Module::new(&self.wasmer_store, &contract.code).unwrap();
            let import_object = imports! {};

            let instance = Instance::new(&module, &import_object).unwrap();

            let check = instance.exports.get_function(M::NAME).unwrap();

            let result = check.call(&[Value::I32(42)]).unwrap();

            dbg!(result);

            todo!()
        } else {
            Err(VMError::UnknownContract)
        }
    }
}
