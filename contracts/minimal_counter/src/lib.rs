// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

#![feature(option_result_unwrap_unchecked)]
#![cfg_attr(target_arch = "wasm32", no_std)]
#![feature(core_intrinsics, lang_items, alloc_error_handler)]

use microkelvin::Store;
use rkyv::{Archive, Deserialize, Serialize};
use vm_proto::{AbiStore, Apply, Execute, Query, Transaction};

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

impl Query for ReadCount {
    const NAME: &'static str = "read";
    type Return = u32;
}

#[derive(Archive, Serialize, Debug, Deserialize)]
pub struct Increment(u32);

impl Transaction for Increment {
    const NAME: &'static str = "incr";
    type Return = ();
}

impl<S> Execute<ReadCount, S> for Counter
where
    S: Store,
{
    fn execute(
        archived_self: &Self::Archived,
        _: &<ReadCount as Archive>::Archived,
        _: &S,
    ) -> <ReadCount as Query>::Return {
        archived_self.value.into()
    }
}

impl<S> Apply<Increment, S> for Counter
where
    S: Store,
{
    fn apply(
        &mut self,
        t: &<Increment as Archive>::Archived,
        _: &S,
    ) -> <Increment as Transaction>::Return {
        let unarchived: u32 = t.0.into();
        self.value += unarchived;
    }
}

#[no_mangle]
unsafe fn read(
    s: *const <Counter as Archive>::Archived,
    q: *const <ReadCount as Archive>::Archived,
    _ret: *mut <<ReadCount as Query>::Return as Archive>::Archived,
) {
    Counter::execute(&*s, &*q, &AbiStore);
    todo!()
}

#[no_mangle]
unsafe fn incr(
    s: *mut <Counter as Archive>::Archived,
    t: *const <Increment as Archive>::Archived,
    _ret: *mut <<Increment as Transaction>::Return as Archive>::Archived,
) {
    let mut store = AbiStore;
    let mut de_state = (&*s).deserialize(&mut store).unwrap_unchecked();
    Counter::apply(&mut de_state, &*t, &AbiStore);
    todo!()
}
