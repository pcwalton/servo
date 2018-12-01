/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use atomic_refcell::AtomicRefCell;
use script_layout_interface::StyleData;

#[repr(C)]
pub struct StyleAndLayoutData {
    /// Data accessed by script_layout_interface. This must be first to allow
    /// casting between StyleAndLayoutData and StyleData.
    pub style_data: StyleData,
    /// The layout data associated with a node.
    pub layout_data: AtomicRefCell<LayoutData>,
}

impl StyleAndLayoutData {
    pub fn new() -> Self {
        Self {
            style_data: StyleData::new(),
            layout_data: AtomicRefCell::new(LayoutData::new()),
        }
    }
}

/// Data that layout associates with a node.
#[repr(C)]
pub struct LayoutData;

impl LayoutData {
    /// Creates new layout data.
    pub fn new() -> LayoutData {
        LayoutData
    }
}
