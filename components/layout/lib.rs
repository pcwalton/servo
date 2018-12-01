/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![deny(unsafe_code)]

#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate html5ever;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde;

mod block;
pub mod construct;
pub mod context;
pub mod data;
pub mod flow;
mod flow_list;
pub mod flow_ref;
mod fragment;
mod model;
pub mod opaque_node;
pub mod parallel;
pub mod query;
pub mod sequential;
pub mod traversal;
pub mod wrapper;

// For unit tests:
pub use self::data::LayoutData;
pub use crate::fragment::Fragment;
pub use crate::fragment::SpecificFragmentInfo;

// We can't use servo_arc for everything in layout, because the Flow stuff uses
// weak references.
use servo_arc::Arc as ServoArc;
