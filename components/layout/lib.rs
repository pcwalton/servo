/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![deny(unsafe_code)]

#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate html5ever;
#[macro_use]
extern crate serde;

pub mod construct;
pub mod context;
pub mod data;
mod model;
pub mod opaque_node;
pub mod query;
pub mod wrapper;

// For unit tests:
pub use self::data::LayoutData;

// We can't use servo_arc for everything in layout, because the Flow stuff uses
// weak references.
use servo_arc::Arc as ServoArc;
