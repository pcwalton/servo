/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::flow::{Flow, FlowClass};
use crate::flow_ref::FlowRef;
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use serde::ser::{Serialize, SerializeSeq, Serializer};
use serde_json::{to_value, Map, Value};
use std::collections::{linked_list, LinkedList};
use std::ops::Deref;
use std::sync::Arc;

/// This needs to be reworked now that we have dynamically-sized types in Rust.
/// Until then, it's just a wrapper around LinkedList.
pub struct FlowList {
    flows: LinkedList<FlowRef>,
}

impl Serialize for FlowList {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut serializer = serializer.serialize_seq(Some(self.len()))?;
        for f in self.iter() {
            let f = f.read();
            let mut flow_val = Map::new();
            flow_val.insert("class".to_owned(), to_value(f.class()).unwrap());
            let data = match f.class() {
                FlowClass::Block => to_value(f.as_block()).unwrap(),
                FlowClass::Inline => to_value(f.as_inline()).unwrap(),
                FlowClass::Table => to_value(f.as_table()).unwrap(),
                FlowClass::TableWrapper => to_value(f.as_table_wrapper()).unwrap(),
                FlowClass::TableRowGroup => to_value(f.as_table_rowgroup()).unwrap(),
                FlowClass::TableRow => to_value(f.as_table_row()).unwrap(),
                FlowClass::TableCell => to_value(f.as_table_cell()).unwrap(),
                FlowClass::Flex => to_value(f.as_flex()).unwrap(),
                FlowClass::ListItem |
                FlowClass::TableColGroup |
                FlowClass::TableCaption |
                FlowClass::Multicol |
                FlowClass::MulticolColumn => {
                    Value::Null // Not implemented yet
                },
            };
            flow_val.insert("data".to_owned(), data);
            serializer.serialize_element(&flow_val)?;
        }
        serializer.end()
    }
}

pub struct FlowListIterator<'a> {
    it: linked_list::Iter<'a, FlowRef>,
}

impl FlowList {
    /// Add an element last in the list
    ///
    /// O(1)
    pub fn push_back(&mut self, new_tail: FlowRef) {
        self.flows.push_back(new_tail);
    }

    pub fn push_back_arc(&mut self, new_head: Arc<RwLock<dyn Flow>>) {
        self.flows.push_back(FlowRef::new(new_head));
    }

    pub fn back(&self) -> Option<RwLockReadGuard<dyn Flow>> {
        self.flows.back().map(|x| x.read())
    }

    /// Add an element first in the list
    ///
    /// O(1)
    pub fn push_front(&mut self, new_head: FlowRef) {
        self.flows.push_front(new_head);
    }

    pub fn push_front_arc(&mut self, new_head: Arc<RwLock<dyn Flow>>) {
        self.flows.push_front(FlowRef::new(new_head));
    }

    pub fn pop_front_arc(&mut self) -> Option<Arc<RwLock<dyn Flow>>> {
        self.flows.pop_front().map(FlowRef::into_arc)
    }

    pub fn front(&self) -> Option<RwLockReadGuard<dyn Flow>> {
        self.flows.front().map(|x| x.read())
    }

    /// Create an empty list
    #[inline]
    pub fn new() -> FlowList {
        FlowList {
            flows: LinkedList::new(),
        }
    }

    /// Provide a forward iterator.
    #[inline]
    pub fn iter<'a>(&'a self) -> FlowListIterator {
        FlowListIterator {
            it: self.flows.iter(),
        }
    }

    /// Provides a caching random-access iterator that yields mutable references. This is
    /// guaranteed to perform no more than O(n) pointer chases.
    #[inline]
    pub fn random_access(&mut self) -> FlowListRandomAccess {
        let length = self.flows.len();
        FlowListRandomAccess {
            iterator: self.flows.iter(),
            cache: Vec::with_capacity(length),
        }
    }

    /// O(1)
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.flows.is_empty()
    }

    /// O(1)
    #[inline]
    pub fn len(&self) -> usize {
        self.flows.len()
    }

    #[inline]
    pub fn split_off(&mut self, i: usize) -> Self {
        FlowList {
            flows: self.flows.split_off(i),
        }
    }
}

impl<'a> DoubleEndedIterator for FlowListIterator<'a> {
    fn next_back(&mut self) -> Option<&'a FlowRef> {
        self.it.next_back()
    }
}

impl<'a> Iterator for FlowListIterator<'a> {
    type Item = &'a FlowRef;
    #[inline]
    fn next(&mut self) -> Option<&'a FlowRef> {
        self.it.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.it.size_hint()
    }
}

/// A caching random-access iterator that yields mutable references. This is guaranteed to perform
/// no more than O(n) pointer chases.
pub struct FlowListRandomAccess<'a> {
    iterator: linked_list::Iter<'a, FlowRef>,
    cache: Vec<FlowRef>,
}

impl<'a> FlowListRandomAccess<'a> {
    pub fn get<'b>(&'b mut self, index: usize) -> &'b FlowRef {
        while index >= self.cache.len() {
            match self.iterator.next() {
                None => panic!("Flow index out of range!"),
                Some(next_flow) => self.cache.push((*next_flow).clone()),
            }
        }
        &self.cache[index]
    }
}
