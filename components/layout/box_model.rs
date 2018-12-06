/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use app_units::Au;
use euclid::Point2D;
use style::logical_geometry::LogicalRect;

pub struct BoxModelData {
    /// The position and size of the border box of this fragment relative to the nearest ancestor
    /// stacking context.
    pub border_box: LogicalRect<Au>,
}

pub trait BoxModelComponent {
    fn get(&self, entity: Entity) -> &BoxModelData;
    fn get_mut(&mut self, entity: Entity) -> &mut BoxModelData;
}
