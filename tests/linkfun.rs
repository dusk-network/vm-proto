use vm_proto::*;

use funlink::{FunLink, Pop, Push};

const CODE: &'static [u8] =
    include_bytes!("../contracts/funlink/target/wasm32-unknown-unknown/release/funlink.wasm");

const N: i32 = 1;

#[test]
fn contract_standalone() {
    let mut fun = FunLink::new();

    assert_eq!(fun.apply(Pop), None);

    for i in 0..N {
        fun.apply(Push(i));
    }

    for i in 0..N {
        assert_eq!(fun.apply(Pop), Some(N - i - 1));
    }

    assert_eq!(fun.apply(Pop), None);
}

#[test]
fn query_deployed_contract() -> Result<(), Box<dyn std::error::Error>> {
    let mut state = State::default();
    let fun = FunLink::new();

    let id = state.deploy(fun, CODE)?;

    assert_eq!(state.apply(id, Pop)?, None);

    for i in 0..N {
        state.apply(id, Push(i))?;
    }

    for i in 0..N {
        assert_eq!(state.apply(id, Pop)?, Some(N - i - 1))
    }

    assert_eq!(state.apply(id, Pop)?, None);

    Ok(())
}
