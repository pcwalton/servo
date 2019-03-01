/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Reference-counted pointers to flows.
//!
//! Eventually, with dynamically sized types in Rust, much of this code will
//! be superfluous. This design is largely duplicating logic of Arc<T> and
//! Weak<T>; please see comments there for details.

use crate::flow::Flow;
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::ops::Deref;
use std::sync::{Arc, Weak};

#[derive(Clone, Debug)]
pub struct FlowRef(Arc<RwLock<dyn Flow>>);

impl FlowRef {
    /// `FlowRef`s can only be made available to the traversal code.
    /// See https://github.com/servo/servo/issues/14014 for more details.
    pub fn new(mut r: Arc<RwLock<dyn Flow>>) -> Self {
        // This assertion checks that this `FlowRef` does not alias normal `Arc`s.
        // If that happens, we're in trouble.
        assert!(Arc::get_mut(&mut r).is_some());
        FlowRef(r)
    }

    pub fn downgrade(this: &FlowRef) -> WeakFlowRef {
        WeakFlowRef(Arc::downgrade(&this.0))
    }

    pub fn into_arc(mut this: FlowRef) -> Arc<RwLock<dyn Flow>> {
        this.0
    }

    pub fn read(&self) -> RwLockReadGuard<dyn Flow> {
        // FIXME(pcwalton): Don't use `read_recursive()`!
        self.0.try_read_recursive().unwrap()
    }

    pub fn write(&self) -> RwLockWriteGuard<dyn Flow> {
        self.0.try_write().unwrap()
    }
}

#[derive(Clone, Debug)]
pub struct WeakFlowRef(Weak<RwLock<dyn Flow>>);

impl WeakFlowRef {
    pub fn upgrade(&self) -> Option<FlowRef> {
        self.0.upgrade().map(FlowRef)
    }
}
