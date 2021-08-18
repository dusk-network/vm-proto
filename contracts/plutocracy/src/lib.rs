// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use std::pin::Pin;

use rkyv::{
    ser::{serializers::WriteSerializer, Serializer},
    Archive, Serialize,
};
use vm_proto::{Apply, Contract, Method, Query};

#[derive(Archive, Serialize, Debug, Default)]
pub struct Plutocracy {
    treasury: u64,
}

impl Plutocracy {
    pub fn new() -> Self {
        Plutocracy { treasury: 0 }
    }
}

impl Contract for Plutocracy {
    fn code() -> &'static [u8] {
        include_bytes!(
            "../target/wasm32-unknown-unknown/release/plutocracy.wasm"
        )
    }
}

#[derive(Archive, Serialize, Debug)]
pub struct TotalSupply;

impl Method for TotalSupply {
    const NAME: &'static str = "total_supply";
    type Return = u64;
}

impl Query<TotalSupply> for Plutocracy {
    fn query(&self, _arg: &TotalSupply) -> u64 {
        self.treasury
    }
}

#[derive(Archive, Serialize, Debug)]
pub struct Mint {
    pub amount: u64,
}

impl Method for Mint {
    const NAME: &'static str = "mint";
    type Return = ();
}

impl Apply<Mint> for Plutocracy {
    fn apply(mut self: Pin<&mut Self>, mint: &Mint) {
        self.treasury += mint.amount
    }
}

// to autogenerate

#[no_mangle]
fn total_supply(
    s: &Plutocracy,
    q: &TotalSupply,
    r: &mut <TotalSupply as Method>::Return,
) {
    *r = s.query(q);
}

#[no_mangle]
fn mint(s: Pin<&mut Plutocracy>, t: &Mint, r: &mut <Mint as Method>::Return) {
    *r = s.apply(t);
}
