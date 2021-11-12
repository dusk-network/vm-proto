// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.
#![no_std]

mod linked_list;
use linked_list::LinkedList;

use rkyv::{Archive, Deserialize, Infallible, Serialize};
use vm_proto::{Apply, Method, Scratch};

#[derive(Archive, Serialize, Default, Deserialize)]
pub struct FunLink {
    list: LinkedList<i32, ()>,
}

impl FunLink {
    pub fn new() -> Self {
        Default::default()
    }
}

#[derive(Archive, Serialize, Deserialize, Debug)]
pub struct Push(pub i32);

impl Method for Push {
    const NAME: &'static str = "push";
    type Return = ();
}

impl Apply<Push> for FunLink {
    fn apply(&mut self, arg: Push) {
        self.list.push(arg.0)
    }
}

#[derive(Archive, Serialize, Deserialize, Debug)]
pub struct Pop;

impl Method for Pop {
    const NAME: &'static str = "pop";
    type Return = Option<i32>;
}

impl Apply<Pop> for FunLink {
    fn apply(&mut self, _: Pop) -> Option<i32> {
        vm_proto::abi::debug("yolo");
        self.list.pop()
    }
}

// to autogenerate

#[no_mangle]
fn pop(
    s: &<FunLink as Archive>::Archived,
    t: &<Pop as Archive>::Archived,
) -> Scratch {
    let mut de: FunLink = s.deserialize(&mut Infallible).expect("infallible");
    let arg: Pop = t.deserialize(&mut Infallible).expect("infallible");
    let result = FunLink::apply(&mut de, arg);
    let new_state = microkelvin::Portal::put(&de);
    Scratch::write(&(new_state, result))
}

#[no_mangle]
fn push(
    s: &<FunLink as Archive>::Archived,
    t: &<Push as Archive>::Archived,
) -> Scratch {
    let mut de: FunLink = s.deserialize(&mut Infallible).expect("infallible");
    let arg: Push = t.deserialize(&mut Infallible).expect("infallible");
    let result = FunLink::apply(&mut de, arg);
    let new_state = microkelvin::Portal::put(&de);
    Scratch::write(&(new_state, result))
}
