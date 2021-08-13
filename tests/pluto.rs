use vm_proto::*;

use plutocracy::{Mint, Plutocracy, TotalSupply};

#[test]
fn contract_standalone() {
    let mut pluto = Plutocracy::new("The real world".into());

    assert_eq!(pluto.query(&TotalSupply), 0);

    pluto.apply(&Mint { amount: 100 });

    assert_eq!(pluto.query(&TotalSupply), 100);
}

#[test]
fn query_deployed_contract() -> Result<(), Box<dyn std::error::Error>> {
    let mut state = State::default();
    let mut pluto = Plutocracy::new("Fuck around and find out".into());
    pluto.apply(&Mint { amount: 255 });

    let id = state.deploy(pluto)?;

    assert_eq!(state.query(id, &TotalSupply).unwrap(), 255);

    Ok(())
}

#[ignore]
#[test]
fn transact_deployed_contract() -> Result<(), Box<dyn std::error::Error>> {
    let mut state = State::default();
    let pluto = Plutocracy::new("Our bright future".into());
    let id = state.deploy(pluto)?;

    assert_eq!(state.query(id, &TotalSupply)?, 0);

    state.apply(id, &Mint { amount: 100 })?;

    assert_eq!(state.query(id, &TotalSupply).unwrap(), 100);

    Ok(())
}
