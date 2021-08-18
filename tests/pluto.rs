use std::pin::Pin;

use vm_proto::*;

use plutocracy::{Mint, Plutocracy, TotalSupply};

#[test]
fn contract_standalone() {
    let mut pluto = Plutocracy::new();

    assert_eq!(pluto.query(&TotalSupply), 0);

    Pin::new(&mut pluto).apply(&Mint { amount: 100 });

    assert_eq!(pluto.query(&TotalSupply), 100);
}

#[test]
fn query_deployed_contract() -> Result<(), Box<dyn std::error::Error>> {
    let n = 201;

    let mut state = State::default();
    let mut pluto = Plutocracy::new();
    Pin::new(&mut pluto).apply(&Mint { amount: n });

    let id = state.deploy(pluto)?;

    assert_eq!(state.query(id, &TotalSupply).unwrap(), n);

    Ok(())
}

#[test]
fn transact_deployed_contract() -> Result<(), Box<dyn std::error::Error>> {
    let mut state = State::default();
    let pluto = Plutocracy::new();
    let id = state.deploy(pluto)?;

    assert_eq!(state.query(id, &TotalSupply)?, 0);

    state.apply(id, &Mint { amount: 100 })?;

    assert_eq!(state.query(id, &TotalSupply).unwrap(), 100);

    Ok(())
}
