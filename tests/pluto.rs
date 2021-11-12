use vm_proto::*;

use microkelvin::{Portal, Storage};
use plutocracy::{Mint, Plutocracy, TotalSupply};
use rkyv::{Archive, Serialize};

const CODE: &'static [u8] =
    include_bytes!("../contracts/plutocracy/target/wasm32-unknown-unknown/release/plutocracy.wasm");

fn query_archived<S, Q>(state: &S, query: &Q) -> Q::Return
where
    Q: Archive + Serialize<Storage> + Method,
    S: Archive + Serialize<Storage> + Query<Q>,
{
    let stored = Portal::put(state);
    let archived = Portal::get(stored);

    // for test only, arguments should generally not be stored
    let stored_arg = Portal::put(query);
    let archived_arg = Portal::get(stored_arg);

    S::query(archived, archived_arg)
}

#[test]
fn contract_standalone() {
    let mut pluto = Plutocracy::new();

    assert_eq!(query_archived(&pluto, &TotalSupply), 0);

    pluto.apply(Mint { amount: 100 });

    assert_eq!(query_archived(&pluto, &TotalSupply), 100);
}

#[test]
fn query_deployed_contract() -> Result<(), Box<dyn std::error::Error>> {
    let n = 201;

    let mut state = State::default();
    let mut pluto = Plutocracy::new();
    pluto.apply(Mint { amount: n });

    let id = state.deploy(pluto, CODE)?;

    assert_eq!(state.query(id, &TotalSupply).unwrap(), n);

    Ok(())
}

#[test]
fn transact_deployed_contract() -> Result<(), Box<dyn std::error::Error>> {
    let mut state = State::default();
    let pluto = Plutocracy::new();
    let id = state.deploy(pluto, CODE)?;

    assert_eq!(state.query(id, &TotalSupply)?, 0);

    state.apply(id, Mint { amount: 100 })?;

    assert_eq!(state.query(id, &TotalSupply).unwrap(), 100);

    Ok(())
}
