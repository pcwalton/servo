/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Borders, padding, and margins.

use app_units::Au;
use std::fmt;

/// A collapsible margin. See CSS 2.1 § 8.3.1.
#[derive(Clone, Copy, Debug)]
pub struct AdjoiningMargins {
    /// The value of the greatest positive margin.
    pub most_positive: Au,

    /// The actual value (not the absolute value) of the negative margin with the largest absolute
    /// value. Since this is not the absolute value, this is always zero or negative.
    pub most_negative: Au,
}

/// Intrinsic inline-sizes, which consist of minimum and preferred.
#[derive(Clone, Copy, Serialize)]
pub struct IntrinsicISizes {
    /// The *minimum inline-size* of the content.
    pub minimum_inline_size: Au,
    /// The *preferred inline-size* of the content.
    pub preferred_inline_size: Au,
}

impl fmt::Debug for IntrinsicISizes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "min={:?}, pref={:?}",
            self.minimum_inline_size, self.preferred_inline_size
        )
    }
}

/// A min-size and max-size constraint. The constructor has a optional `border`
/// parameter, and when it is present the constraint will be subtracted. This is
/// used to adjust the constraint for `box-sizing: border-box`, and when you do so
/// make sure the size you want to clamp is intended to be used for content box.
#[derive(Clone, Copy, Debug, Serialize)]
pub struct SizeConstraint {
    min_size: Au,
    max_size: Option<Au>,
}
