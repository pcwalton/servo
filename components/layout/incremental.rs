/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use flow::{mod, Flow, ImmutableFlowUtils};

use std::fmt;
use std::sync::Arc;
use style::computed_values::position;
use style::ComputedValues;

bitflags! {
    #[doc = "Individual layout actions that may be necessary after restyling."]
    flags RestyleDamage: u8 {
        #[doc = "Repaint the node itself."]
        #[doc = "Currently unused; need to decide how this propagates."]
        static Repaint = 0x01,

        #[doc = "Recompute intrinsic inline_sizes (minimum and preferred)."]
        #[doc = "Propagates down the flow tree because the computation is"]
        #[doc = "bottom-up."]
        static BubbleISizes = 0x02,

        #[doc = "Recompute actual inline-sizes and block-sizes, only taking out-of-flow children \
                 into account. \
                 Propagates up the flow tree because the computation is top-down."]
        static Reposition = 0x04,

        #[doc = "Recompute actual inline_sizes and block_sizes."]
        #[doc = "Propagates up the flow tree because the computation is"]
        #[doc = "top-down."]
        static Reflow = 0x08,

        #[doc = "Reconstruct the flow."]
        static ReconstructFlow = 0x10
    }
}

bitflags! {
    flags SpecialRestyleDamage: u8 {
        static ReflowEntireDocument = 0x01,
    }
}

impl RestyleDamage {
    /// Supposing a flow has the given `position` property and this damage, returns the damage that
    /// the *parent* of this flow should have.
    pub fn damage_for_parent(self, child_positioning: position::T) -> RestyleDamage {
        match child_positioning {
            position::absolute => self & (Repaint | Reposition),
            _ => self & (Repaint | Reflow | Reposition),
        }
    }

    /// Supposing the *parent* of a flow with the given `position` property has this damage,
    /// returns the damage that this flow should have.
    pub fn damage_for_child(self, child_positioning: position::T) -> RestyleDamage {
        match child_positioning {
            position::absolute => self & Repaint,
            _ => {
                // TODO(pcwalton): Take floatedness into account.
                self & (Repaint | Reflow | Reposition)
            }
        }
    }
}

impl fmt::Show for RestyleDamage {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::FormatError> {
        let mut first_elem = true;

        let to_iter =
            [ (Repaint,         "Repaint")
            , (BubbleISizes,    "BubbleISizes")
            , (Reposition,      "Reposition")
            , (Reflow,          "Reflow")
            , (ReconstructFlow, "ReconstructFlow")
            ];

        for &(damage, damage_str) in to_iter.iter() {
            if self.contains(damage) {
                if !first_elem { try!(write!(f, " | ")); }
                try!(write!(f, "{}", damage_str));
                first_elem = false;
            }
        }

        if first_elem {
            try!(write!(f, "NoDamage"));
        }

        Ok(())
    }
}

// NB: We need the braces inside the RHS due to Rust #8012.  This particular
// version of this macro might be safe anyway, but we want to avoid silent
// breakage on modifications.
macro_rules! add_if_not_equal(
    ($old:ident, $new:ident, $damage:ident,
     [ $($effect:ident),* ], [ $($style_struct_getter:ident.$name:ident),* ]) => ({
        if $( ($old.$style_struct_getter().$name != $new.$style_struct_getter().$name) )||* {
            $damage.insert($($effect)|*);
        }
    })
)

pub fn compute_damage(old: &Option<Arc<ComputedValues>>, new: &ComputedValues) -> RestyleDamage {
    let old: &ComputedValues =
        match old.as_ref() {
            None => return RestyleDamage::all(),
            Some(cv) => &**cv,
        };

    let mut damage = RestyleDamage::empty();

    // This checks every CSS property, as enumerated in
    // impl<'self> CssComputedStyle<'self>
    // in src/support/netsurfcss/rust-netsurfcss/netsurfcss.rc.

    // FIXME: We can short-circuit more of this.

    add_if_not_equal!(old, new, damage,
                      [ Repaint ], [
        get_color.color, get_background.background_color,
        get_border.border_top_color, get_border.border_right_color,
        get_border.border_bottom_color, get_border.border_left_color
    ]);

    add_if_not_equal!(old, new, damage,
                      [ Repaint, Reposition ], [
        get_positionoffsets.top, get_positionoffsets.left,
        get_positionoffsets.right, get_positionoffsets.bottom
    ]);

    add_if_not_equal!(old, new, damage,
                      [ Repaint, BubbleISizes, Reposition, Reflow ], [
        get_border.border_top_width, get_border.border_right_width,
        get_border.border_bottom_width, get_border.border_left_width,
        get_margin.margin_top, get_margin.margin_right,
        get_margin.margin_bottom, get_margin.margin_left,
        get_padding.padding_top, get_padding.padding_right,
        get_padding.padding_bottom, get_padding.padding_left,
        get_box.width, get_box.height,
        get_font.font_family, get_font.font_size, get_font.font_style, get_font.font_weight,
        get_inheritedtext.text_align, get_text.text_decoration, get_inheritedbox.line_height
    ]);

    add_if_not_equal!(old, new, damage,
                      [ Repaint, BubbleISizes, Reposition, Reflow, ReconstructFlow ],
                      [ get_box.float, get_box.display, get_box.position ]);

    // FIXME: test somehow that we checked every CSS property

    if damage.intersects(Repaint | Reflow) {
        println!("found CSS damage!");
    }

    damage
}

pub trait LayoutDamageComputation {
    fn find_non_abspos_damaged_things(self);
    fn find_damaged_things(self);
    fn compute_layout_damage(self) -> SpecialRestyleDamage;
    fn reflow_entire_document(self);
}

impl<'a> LayoutDamageComputation for &'a mut Flow+'a {
    fn find_non_abspos_damaged_things(self) {
        let positioning = self.positioning();
        let self_base = flow::mut_base(self);
        for kid in self_base.children.iter_mut() {
            kid.find_non_abspos_damaged_things();
        }
    }

    fn find_damaged_things(self) {
        let positioning = self.positioning();
        if flow::mut_base(self).restyle_damage.intersects(Reflow | Reposition) {
            println!("damage found -- impacted by floats? {}!",
                     flow::base(self).flags.impacted_by_floats());
            self.dump();
        }
        let self_base = flow::mut_base(self);
        for kid in self_base.children.iter_mut() {
            kid.find_damaged_things();
        }
    }

    fn compute_layout_damage(self) -> SpecialRestyleDamage {
        let mut special_damage = SpecialRestyleDamage::empty();

        {
            let positioning = self.positioning();
            let self_base = flow::mut_base(self);
            for kid in self_base.children.iter_mut() {
                let child_positioning = kid.positioning();
                /*println!("abspos={} before damage={}",
                         positioning == position::absolute,
                         self_base.restyle_damage);*/
                flow::mut_base(kid).restyle_damage
                                   .insert(self_base.restyle_damage
                                                    .damage_for_child(child_positioning));
                special_damage.insert(kid.compute_layout_damage());
                self_base.restyle_damage
                         .insert(flow::base(kid).restyle_damage
                                                .damage_for_parent(child_positioning));
                /*println!("abspos={} after damage={}",
                         positioning == position::absolute,
                         self_base.restyle_damage);*/
            }

            /*if self_base.restyle_damage != RestyleDamage::empty() &&
                    self_base.restyle_damage != Repaint &&
                    self_base.restyle_damage != (Repaint | BubbleISizes | Reposition | Reflow) {
                println!("flow was damaged: {}", self_base.restyle_damage);
            }*/
        }

        let self_base = flow::base(self);
        if self.is_float() && self_base.restyle_damage.intersects(Reposition | Reflow) {
            special_damage.insert(ReflowEntireDocument);
        }

        special_damage
    }

    fn reflow_entire_document(self) {
        let self_base = flow::mut_base(self);
        self_base.restyle_damage.insert(Repaint | BubbleISizes | Reflow | Reposition);
        for kid in self_base.children.iter_mut() {
            kid.reflow_entire_document();
        }
    }
}

