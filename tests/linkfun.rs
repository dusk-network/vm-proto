use std::pin::Pin;

use vm_proto::*;

use funlink::{FunLink, Pop, Push};

const CODE: &'static [u8] =
    include_bytes!("../contracts/funlink/target/wasm32-unknown-unknown/release/funlink.wasm");

#[test]
fn contract_standalone() {
    let mut fun = FunLink::new();

    assert_eq!(Pin::new(&mut fun).apply(&Pop), Ok(None));

    Pin::new(&mut fun).apply(&Push(99));

    assert_eq!(Pin::new(&mut fun).apply(&Pop), Ok(Some(99)));
    assert_eq!(Pin::new(&mut fun).apply(&Pop), Ok(None));
}

#[test]
fn query_deployed_contract() -> Result<(), Box<dyn std::error::Error>> {
    let n = 201;

    let mut state = State::default();
    let mut fun = FunLink::new();

    let id = state.deploy(fun, CODE)?;

    Ok(())
}
