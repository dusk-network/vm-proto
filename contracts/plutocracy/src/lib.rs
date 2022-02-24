// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.
#![no_std]

use rkyv::{Archive, Deserialize, Infallible, Serialize};
use vm_proto::{Apply, Method, Query, Scratch};

#[derive(Archive, Serialize, Debug, Default, Deserialize)]
pub struct Plutocracy {
    treasury: u64,
}

impl Plutocracy {
    pub fn new() -> Self {
        Plutocracy { treasury: 0 }
    }
}

#[derive(Archive, Serialize, Deserialize, Debug)]
pub struct TotalSupply;

impl Method for TotalSupply {
    const NAME: &'static str = "total_supply";
    type Return = u64;
}

impl Query<TotalSupply> for Plutocracy {
    fn query(
        archived: &Self::Archived,
        _arg: &<TotalSupply as Archive>::Archived,
    ) -> u64 {
        let dearchived: u64 = archived.treasury.into();
        dearchived
    }
}

#[derive(Archive, Serialize, Deserialize, Debug)]
pub struct Mint {
    pub amount: u64,
}

impl Method for Mint {
    const NAME: &'static str = "mint";
    type Return = ();
}

impl Apply<Mint> for Plutocracy {
    fn apply(&mut self, mint: Mint) {
        self.treasury += mint.amount
    }
}

// to autogenerate

#[no_mangle]
fn total_supply(
    s: &<Plutocracy as Archive>::Archived,
    q: &<TotalSupply as Archive>::Archived,
) -> Scratch {
    Scratch::write(&Plutocracy::query(s, q))
}

#[no_mangle]
fn mint(
    s: &<Plutocracy as Archive>::Archived,
    t: &<Mint as Archive>::Archived,
) -> Scratch {
    let mut de: Plutocracy =
        s.deserialize(&mut Infallible).expect("infallible");
    let arg: Mint = t.deserialize(&mut Infallible).expect("infallible");
    let result = Plutocracy::apply(&mut de, arg);
    let new_state = microkelvin::Portal::put(&de);
    Scratch::write(&(new_state, result))
}
