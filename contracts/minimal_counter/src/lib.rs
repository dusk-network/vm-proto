// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

#![cfg_attr(target_arch = "wasm32", no_std)]
#![feature(core_intrinsics, lang_items, alloc_error_handler)]

use rkyv::{Archive, Deserialize, Infallible, Serialize};
use vm_proto::{Apply, Method, Query, Scratch};

#[derive(Clone, Debug, Archive, Deserialize, Serialize)]
pub struct Counter {
    value: u32,
}

impl Counter {
    pub fn new(value: u32) -> Self {
        Counter { value }
    }
}

#[derive(Archive, Serialize, Debug)]
pub struct ReadCount;

impl Method for ReadCount {
    const NAME: &'static str = "read";
    type Return = u32;
}

#[derive(Archive, Serialize, Debug, Deserialize)]
pub struct Increment(u32);

impl Method for Increment {
    const NAME: &'static str = "incr";
    type Return = ();
}

impl Query<ReadCount> for Counter {
    fn query(
        archived: &Self::Archived,
        _: &<ReadCount as Archive>::Archived,
    ) -> <ReadCount as Method>::Return {
        archived.value.into()
    }
}

impl Apply<Increment> for Counter {
    fn apply(&mut self, t: Increment) -> <Increment as Method>::Return {
        let unarchived: u32 = t.0.into();
        self.value += unarchived;
    }
}

#[no_mangle]
fn read(
    s: &<Counter as Archive>::Archived,
    q: &<ReadCount as Archive>::Archived,
) -> Scratch {
    Scratch::write(&Counter::query(s, q))
}

#[no_mangle]
fn incr(
    s: &<Counter as Archive>::Archived,
    t: &<Increment as Archive>::Archived,
) -> Scratch {
    let mut de: Counter = s.deserialize(&mut Infallible).expect("infallible");
    let arg: Increment = t.deserialize(&mut Infallible).expect("infallible");
    let result = Counter::apply(&mut de, arg);
    let new_state = microkelvin::Portal::put(&de);
    Scratch::write(&(new_state, result))
}
