/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Constructs display lists from boxes.

use layout::context::LayoutContext;

use gfx::render_task::RenderLayer;
use gfx;
use servo_util::smallvec::SmallVec0;
use style;

/// Manages the information needed to construct the display list.
///
/// FIXME(pcwalton): Throw more information in here instead of threading it around in parameters.
pub struct DisplayListBuilder<'a> {
    ctx: &'a LayoutContext,

    /// A list of render layers that we've built up, root layer not included.
    layers: SmallVec0<RenderLayer>,
}

//
// Miscellaneous useful routines
//

/// Allows a CSS color to be converted into a graphics color.
pub trait ToGfxColor {
    /// Converts a CSS color to a graphics color.
    fn to_gfx_color(&self) -> gfx::color::Color;
}

impl ToGfxColor for style::computed_values::RGBA {
    fn to_gfx_color(&self) -> gfx::color::Color {
        gfx::color::rgba(self.red, self.green, self.blue, self.alpha)
    }
}

