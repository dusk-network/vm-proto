// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use microkelvin::{
    Annotation, ArchivedChild, ArchivedCompound, Child, ChildMut, Compound,
    Link, MutableLeaves, Storage, StorageSerializer,
};
use rkyv::{Archive, Deserialize, Serialize};

#[derive(Clone, Archive, Serialize, Deserialize)]
#[archive(bound(serialize = "
  T: Serialize<Storage>,
  A: Annotation<T>,
  __S: StorageSerializer"))]
#[archive(bound(deserialize = "
  A: Archive + Clone,
  T::Archived: Deserialize<T, __D>,
  A::Archived: Deserialize<A, __D>,
  __D: Sized"))]
pub enum LinkedList<T, A> {
    Empty,
    Node {
        val: T,
        #[omit_bounds]
        next: Link<Self, A>,
    },
}

impl<T, A> Default for LinkedList<T, A> {
    fn default() -> Self {
        LinkedList::Empty
    }
}

impl<T, A> ArchivedCompound<LinkedList<T, A>, A> for ArchivedLinkedList<T, A>
where
    T: Archive,
    A: Annotation<T>,
{
    fn child(&self, ofs: usize) -> ArchivedChild<LinkedList<T, A>, A> {
        match (self, ofs) {
            (ArchivedLinkedList::Node { val, .. }, 0) => {
                ArchivedChild::Leaf(val)
            }
            (ArchivedLinkedList::Node { next, .. }, 1) => {
                ArchivedChild::Node(next)
            }
            (ArchivedLinkedList::Node { .. }, _) => ArchivedChild::EndOfNode,
            (ArchivedLinkedList::Empty, _) => ArchivedChild::EndOfNode,
        }
    }
}

impl<T, A> Compound<A> for LinkedList<T, A>
where
    T: Archive,
    A: Annotation<T>,
{
    type Leaf = T;

    fn child(&self, ofs: usize) -> Child<Self, A> {
        match (ofs, self) {
            (0, LinkedList::Node { val, .. }) => Child::Leaf(val),
            (1, LinkedList::Node { next, .. }) => Child::Node(next),
            (_, LinkedList::Node { .. }) => Child::EndOfNode,
            (_, LinkedList::Empty) => Child::EndOfNode,
        }
    }

    fn child_mut(&mut self, ofs: usize) -> ChildMut<Self, A> {
        match (ofs, self) {
            (0, LinkedList::Node { val, .. }) => ChildMut::Leaf(val),
            (1, LinkedList::Node { next, .. }) => ChildMut::Node(next),
            (_, LinkedList::Node { .. }) => ChildMut::EndOfNode,
            (_, LinkedList::Empty) => ChildMut::EndOfNode,
        }
    }
}

impl<T, A> MutableLeaves for LinkedList<T, A> where A: Archive + Annotation<T> {}

impl<T, A> LinkedList<T, A>
where
    A: Annotation<T>,
{
    pub fn new() -> Self {
        Default::default()
    }

    pub fn push(&mut self, t: T) {
        match core::mem::take(self) {
            LinkedList::Empty => {
                *self = LinkedList::Node {
                    val: t,
                    next: Link::new(LinkedList::<T, A>::Empty),
                }
            }
            old @ LinkedList::Node { .. } => {
                *self = LinkedList::Node {
                    val: t,
                    next: Link::new(old),
                };
            }
        }
    }

    pub fn pop(&mut self) -> Option<T>
    where
        T: Clone,
        A: Clone,
    {
        match core::mem::take(self) {
            LinkedList::Empty => None,
            LinkedList::Node { val: t, next } => {
                *self = next.unlink();
                Some(t)
            }
        }
    }
}
