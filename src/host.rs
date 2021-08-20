use std::collections::HashMap as Map;
use std::fmt::{Debug, Display};
use std::io;
use std::mem;

use crate::definitions::*;

//use rkyv::de::deserializers::*;
use rkyv::validation::CheckArchiveError;
use rkyv::{
    check_archived_root, ser::serializers::*, ser::Serializer,
    validation::validators::DefaultValidator, Archive,
};
use rkyv::{AlignedVec, Deserialize, Infallible, Serialize};

use thiserror::Error;
use wasmer::{
    imports, CompileError, ExportError, Instance, MemoryError, Module, RuntimeError, Store,
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
    MemoryError(#[from] MemoryError),
    #[error("{0}")]
    IO(#[from] io::Error),
    #[error("{0}")]
    Infallible(#[from] std::convert::Infallible),
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

impl<A, B> From<CheckArchiveError<A, B>> for VMError
where
    A: Display,
    B: Display,
{
    fn from(comp: CheckArchiveError<A, B>) -> Self {
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
    pub fn deploy<State, Code>(&mut self, state: State, code: Code) -> Result<ContractId, VMError>
    where
        State: Debug + Serialize<DefaultSerializer>,
        Code: Into<Vec<u8>>,
    {
        let mut serialize = DefaultSerializer::default();
        let state_ofs = serialize.serialize_value(&state)?;
        let state = serialize.into_serializer().into_inner();

        let instance = ContractInstance {
            code: code.into(),
            state,
            state_ofs: state_ofs as i32,
        };

        let id = instance.id();

        self.map.insert(id, instance);
        Ok(id)
    }

    pub fn query<M>(&self, id: ContractId, arg: &M) -> Result<M::Return, VMError>
    where
        M: Method + Archive + for<'a> Serialize<WriteSerializer<&'a mut [u8]>>,
        M::Return: Archive,
        <M::Return as Archive>::Archived: for<'a> bytecheck::CheckBytes<DefaultValidator<'a>>
            + Deserialize<<M as Method>::Return, Infallible>,
    {
        fn debug(string: &'static str) {
            println!("debug: {}", string)
        }

        let debug_func = Function::new_native(&store, debug);

        if let Some(contract) = self.map.get(&id) {
            let module = Module::new(&self.wasmer_store, &contract.code)?;
            let import_object = imports! {
            "env" => {
                "debug" => debug_import,
            }
            };
            let instance = Instance::new(&module, &import_object).unwrap();
            let function = instance
                .exports
                .get_native_function::<(i32, i32, i32), ()>(M::NAME)?;
            let memory = instance.exports.get_memory("memory")?;

            // Copy the data the contract needs to execute correctly into its memory.

            // this is first the state of the contract, then the argument, and finally allocated space for the return value.

            let (arg_ofs, ret_ofs) = unsafe {
                // Unsafe because the compiler cannot guarantee that no one else is accessing this memory at this time

                let mem_slice = memory.data_unchecked_mut();

                let state = &contract.state;
                let state_len = state.len();
                mem_slice[0..state_len].copy_from_slice(state);

                let remaining_slice = &mut mem_slice[state_len..];

                let state_len = state_len as i32;

                // Write the argument into wasm memory

                let mut serialize = WriteSerializer::new(remaining_slice);
                let arg_ofs = state_len + serialize.serialize_value(arg)? as i32;

                // TODO, make sure we have enough room in the memory for the return value

                (arg_ofs, arg_ofs + mem::size_of::<M>() as i32)
            };

            function.call(contract.state_ofs, arg_ofs, ret_ofs)?;

            unsafe {
                let mem_slice = memory.data_unchecked();
                let ret_ofs = ret_ofs as usize;
                let ret_len = mem::size_of::<<M as Method>::Return>();
                let ret_slice = &mem_slice[ret_ofs..][..ret_len];
                let archived = check_archived_root::<M::Return>(ret_slice)?;
                let a = archived.deserialize(&mut Infallible)?;
                Ok(a)
            }
        } else {
            Err(VMError::UnknownContract)
        }
    }

    pub fn apply<M>(&mut self, id: ContractId, arg: &M) -> Result<M::Return, VMError>
    where
        M: Method + Archive + for<'a> Serialize<WriteSerializer<&'a mut [u8]>>,
        M::Return: Archive,
        <M::Return as Archive>::Archived: for<'a> bytecheck::CheckBytes<DefaultValidator<'a>>
            + Deserialize<<M as Method>::Return, Infallible>,
    {
        if let Some(contract) = self.map.get_mut(&id) {
            let module = Module::new(&self.wasmer_store, &contract.code)?;
            let import_object = imports! {};
            let instance = Instance::new(&module, &import_object).unwrap();
            let function = instance
                .exports
                .get_native_function::<(i32, i32, i32), ()>(M::NAME)?;
            let memory = instance.exports.get_memory("memory")?;

            // Copy the data the contract needs to execute correctly into its memory.

            // this is first the state of the contract, then the argument, and finally allocated space for the return value.

            let state_len = contract.state.len();

            let (arg_ofs, ret_ofs) = unsafe {
                // Unsafe because the compiler cannot guarantee that no one else is accessing this memory at this time

                let mem_slice = memory.data_unchecked_mut();

                mem_slice[0..state_len].copy_from_slice(&contract.state);

                let remaining_slice = &mut mem_slice[state_len..];

                let state_len = state_len as i32;

                // Write the argument into wasm memory

                let mut serialize = WriteSerializer::new(remaining_slice);
                let arg_ofs = state_len + serialize.serialize_value(arg)? as i32;

                // TODO, make sure we have enough room in the memory

                (arg_ofs, arg_ofs + mem::size_of::<M>() as i32)
            };

            function.call(contract.state_ofs, arg_ofs, ret_ofs)?;

            unsafe {
                let mem_slice = memory.data_unchecked();

                contract.state[..].copy_from_slice(&mem_slice[..state_len]);

                let ret_ofs = ret_ofs as usize;
                let ret_len = mem::size_of::<<<M as Method>::Return as Archive>::Archived>();
                let ret_slice = &mem_slice[ret_ofs..][..ret_len];
                let archived = check_archived_root::<<M as Method>::Return>(ret_slice)?;
                let a = archived.deserialize(&mut Infallible)?;
                Ok(a)
            }
        } else {
            Err(VMError::UnknownContract)
        }
    }
}
