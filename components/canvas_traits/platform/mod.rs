/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#[cfg(target_os = "macos")]
pub use crate::platform::macos as default;
#[cfg(not(target_os = "macos"))]
pub use crate::platform::fallback as default;

pub mod fallback;
#[cfg(target_os = "macos")]
pub mod macos;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum NativeSurfaceFormat {
    Rgb,
    Rgba,
}
