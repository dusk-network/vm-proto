use std::collections::HashMap as Map;
use std::fmt::{Debug, Display};
use std::io;
use std::mem;

use crate::definitions::*;

use microkelvin::Store;
use rkyv::validation::CheckArchiveError;
use rkyv::{
    check_archived_root, ser::serializers::*, ser::Serializer,
    validation::validators::DefaultValidator, AlignedVec, Archive, Deserialize, Infallible,
    Serialize,
};

use thiserror::Error;
use wasmer::{
    imports, CompileError, ExportError, Function, ImportObject, Instance, LazyInit, Memory,
    MemoryError, Module, RuntimeError, Store as WasmerStore, WasmerEnv,
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
}

impl ContractInstance {
    fn id(&self) -> ContractId {
        Default::default()
    }
}

#[derive(Debug, Default)]
pub struct State {
    map: Map<ContractId, ContractInstance>,
    wasmer_store: WasmerStore,
}

fn imports(store: &WasmerStore) -> ImportObject {
    let env = TransactionEnv {
        memory: LazyInit::new(),
    };

    fn debug(env: &TransactionEnv, ofs: i32, len: i32) {
        if let Some(mem) = env.memory.get_ref() {
            let data = unsafe { mem.data_unchecked() };
            let string = std::str::from_utf8(&data[ofs as usize..][..len as usize]).unwrap();

            println!("CONTRACT DEBUG {:?}", string)
        } else {
            panic!("no memory no fun")
        }
    }

    imports! {
            "env" => {
                "debug" => Function::new_native_with_env(store, env, debug),
            }
    }
}

#[derive(WasmerEnv, Clone)]
struct TransactionEnv {
    #[wasmer(export)]
    memory: LazyInit<Memory>,
}

impl State {
    pub fn deploy<State, Code, S>(
        &mut self,
        state: State,
        code: Code,
        store: S,
    ) -> Result<ContractId, VMError>
    where
        S: Store,
        Code: Into<Vec<u8>>,
    {	
        let instance = ContractInstance {
            code: code.into(),
            state,
        };

        let stored = store.put(instance);
        let id = stored.ident();

        self.map.insert(id, instance);
        Ok(id)
    }

    pub fn query<Q>(&self, id: ContractId, arg: &Q) -> Result<Q::Return, VMError>
    where
        Q: Query + Archive + for<'a> Serialize<WriteSerializer<&'a mut [u8]>>,
        Q::Return: Archive,
        <Q as Archive>::Archived: for<'a> bytecheck::CheckBytes<DefaultValidator<'a>>
            + Deserialize<<Q as Query>::Return, Infallible>,
    {
        if let Some(contract) = self.map.get(&id) {
            let module = Module::new(&self.wasmer_store, &contract.code)?;
            let instance = Instance::new(&module, &imports(&self.wasmer_store)).unwrap();
            let function = instance
                .exports
                .get_native_function::<(i32, i32, i32), ()>(Q::NAME)?;
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

                (arg_ofs, arg_ofs + mem::size_of::<Q>() as i32)
            };

            function.call(contract.state_ofs, arg_ofs, ret_ofs)?;

            unsafe {
                let mem_slice = memory.data_unchecked();
                let ret_ofs = ret_ofs as usize;
                let ret_len = mem::size_of::<<Q as Query>::Return>();
                let ret_slice = &mem_slice[ret_ofs..][..ret_len];
                let archived = check_archived_root::<Q::Return>(ret_slice)?;
                let a = archived.deserialize(&mut Infallible)?;
                Ok(a)
            }
        } else {
            Err(VMError::UnknownContract)
        }
    }

    pub fn apply<T>(&mut self, id: ContractId, transaction: T) -> Result<T::Return, VMError>
    where
        T: Archive + for<'a> Serialize<WriteSerializer<&'a mut [u8]>> + Transaction,
        T::Return: Archive,
        <T::Return as Archive>::Archived: for<'a> bytecheck::CheckBytes<DefaultValidator<'a>>
            + Deserialize<<T as Transaction>::Return, Infallible>,
    {
        if let Some(contract) = self.map.get_mut(&id) {
            let module = Module::new(&self.wasmer_store, &contract.code)?;
            let instance = Instance::new(&module, &imports(&self.wasmer_store)).unwrap();
            let function = instance
                .exports
                .get_native_function::<(i32, i32, i32), ()>(T::NAME)?;
            let memory = instance.exports.get_memory("memory")?;

            // Copy the data the contract needs to execute correctly into its memory.

            // this is first the state of the contract, then the argument, and finally allocated space for the return value.

            let state_len = contract.state.len();

            let mem_slice = unsafe { memory.data_unchecked_mut() };

            // Write contract state into wasm memory
            mem_slice[0..state_len].copy_from_slice(&contract.state);

            let remaining_slice = &mut mem_slice[state_len..];

            // Write the argument into wasm memory

            let mut serialize = WriteSerializer::new(remaining_slice);
            let arg_ofs = state_len as i32 + serialize.serialize_value(&transaction)? as i32;

            // FIXME: make sure we have enough room in the memory

            let ret_ofs = arg_ofs + mem::size_of::<T>() as i32;

            function.call(contract.state_ofs, arg_ofs, ret_ofs)?;

            // copy the possibly modified state back to the saved contract state

            contract.state[..].copy_from_slice(&mem_slice[..state_len]);

            let ret_ofs = ret_ofs as usize;
            let ret_len = mem::size_of::<<<T as Transaction>::Return as Archive>::Archived>();

            let archived = check_archived_root::<<T as Transaction>::Return>(
                &mem_slice[ret_ofs..][..ret_len],
            )?;

            Ok(archived.deserialize(&mut Infallible).expect("Infallible"))
        } else {
            Err(VMError::UnknownContract)
        }
    }
}
