// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.
#![no_std]

mod linked_list;
use linked_list::LinkedList;
use microkelvin::LinkError;

use core::pin::Pin;
use rkyv::{Archive, Serialize};
use vm_proto::{Apply, Method};

#[derive(Archive, Serialize, Debug, Default)]
pub struct FunLink {
    list: LinkedList<i32, ()>,
}

impl FunLink {
    pub fn new() -> Self {
        Default::default()
    }
}

#[derive(Archive, Serialize, Debug)]
pub struct Push(pub i32);

impl Method for Push {
    const NAME: &'static str = "push";
    type Return = ();
}

impl Apply<Push> for FunLink {
    fn apply(mut self: Pin<&mut Self>, arg: &Push) {
        self.list.push(arg.0)
    }
}

#[derive(Archive, Serialize, Debug)]
pub struct Pop;

impl Method for Pop {
    const NAME: &'static str = "pop";
    type Return = Result<Option<i32>, LinkError>;
}

impl Apply<Pop> for FunLink {
    fn apply(
        mut self: Pin<&mut Self>,
        _pop: &Pop,
    ) -> Result<Option<i32>, LinkError> {
        self.list.pop()
    }
}

// to autogenerate

#[no_mangle]
fn push(s: Pin<&mut FunLink>, t: &Push, r: &mut <Push as Method>::Return) {
    *r = s.apply(t);
}

#[no_mangle]
fn mint(s: Pin<&mut FunLink>, t: &Pop, r: &mut <Pop as Method>::Return) {
    *r = s.apply(t);
}
