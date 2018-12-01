/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! A safe wrapper for DOM nodes that prevents layout from mutating the DOM, from letting DOM nodes
//! escape, and from generally doing anything that it isn't supposed to. This is accomplished via
//! a simple whitelist of allowed operations, along with some lifetime magic to prevent nodes from
//! escaping.
//!
//! As a security wrapper is only as good as its whitelist, be careful when adding operations to
//! this list. The cardinal rules are:
//!
//! 1. Layout is not allowed to mutate the DOM.
//!
//! 2. Layout is not allowed to see anything with `LayoutDom` in the name, because it could hang
//!    onto these objects and cause use-after-free.
//!
//! When implementing wrapper functions, be careful that you do not touch the borrow flags, or you
//! will race and cause spurious thread failure. (Note that I do not believe these races are
//! exploitable, but they'll result in brokenness nonetheless.)
//!
//! Rules of the road for this file:
//!
//! * Do not call any methods on DOM nodes without checking to see whether they use borrow flags.
//!
//!   o Instead of `get_attr()`, use `.get_attr_val_for_layout()`.
//!
//!   o Instead of `html_element_in_html_document()`, use
//!     `html_element_in_html_document_for_layout()`.

#![allow(unsafe_code)]

use atomic_refcell::{AtomicRef, AtomicRefMut};
use crate::data::{LayoutData, StyleAndLayoutData};
use script_layout_interface::wrapper_traits::GetLayoutData;

pub trait LayoutNodeLayoutData {
    /// Similar to borrow_data*, but returns the full PersistentLayoutData rather
    /// than only the style::data::ElementData.
    fn borrow_layout_data(&self) -> Option<AtomicRef<LayoutData>>;
    fn mutate_layout_data(&self) -> Option<AtomicRefMut<LayoutData>>;
    fn flow_debug_id(self) -> usize;
}

impl<T: GetLayoutData> LayoutNodeLayoutData for T {
    fn borrow_layout_data(&self) -> Option<AtomicRef<LayoutData>> {
        self.get_raw_data().map(|d| d.layout_data.borrow())
    }

    fn mutate_layout_data(&self) -> Option<AtomicRefMut<LayoutData>> {
        self.get_raw_data().map(|d| d.layout_data.borrow_mut())
    }

    fn flow_debug_id(self) -> usize {
        0
    }
}

pub trait GetRawData {
    fn get_raw_data(&self) -> Option<&StyleAndLayoutData>;
}

impl<T: GetLayoutData> GetRawData for T {
    fn get_raw_data(&self) -> Option<&StyleAndLayoutData> {
        self.get_style_and_layout_data().map(|opaque| {
            let container = opaque.ptr.as_ptr() as *mut StyleAndLayoutData;
            unsafe { &*container }
        })
    }
}

pub enum TextContent {
    Text(Box<str>),
}

impl TextContent {
    pub fn is_empty(&self) -> bool {
        match *self {
            TextContent::Text(_) => false,
        }
    }
}
