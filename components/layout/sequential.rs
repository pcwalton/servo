/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Implements sequential traversals over the DOM and flow trees.

use app_units::Au;
use crate::context::LayoutContext;
use crate::flow::{Flow, GetBaseFlow, ImmutableFlowUtils};
use crate::fragment::{CoordinateSystem, FragmentBorderBoxIterator};
use crate::traversal::{AssignBSizes, AssignISizes, BubbleISizes};
use crate::traversal::{PostorderFlowTraversal, PreorderFlowTraversal};
use euclid::{Point2D, Vector2D};
use servo_config::opts;
use style::servo::restyle_damage::ServoRestyleDamage;
use webrender_api::LayoutPoint;

/// Run the main layout passes sequentially.
pub fn reflow(root: &mut dyn Flow, layout_context: &LayoutContext) {
    fn doit(
        flow: &mut dyn Flow,
        assign_inline_sizes: AssignISizes,
        assign_block_sizes: AssignBSizes,
    ) {
        if assign_inline_sizes.should_process(flow) {
            assign_inline_sizes.process(flow);
        }

        for kid in flow.mut_base().child_iter_mut() {
            doit(kid, assign_inline_sizes, assign_block_sizes);
        }

        if assign_block_sizes.should_process(flow) {
            assign_block_sizes.process(flow);
        }
    }

    if opts::get().bubble_inline_sizes_separately {
        let bubble_inline_sizes = BubbleISizes {
            layout_context: &layout_context,
        };
        bubble_inline_sizes.traverse(root);
    }

    let assign_inline_sizes = AssignISizes {
        layout_context: &layout_context,
    };
    let assign_block_sizes = AssignBSizes {
        layout_context: &layout_context,
    };

    doit(root, assign_inline_sizes, assign_block_sizes);
}

pub fn iterate_through_flow_tree_fragment_border_boxes(
    root: &mut dyn Flow,
    iterator: &mut dyn FragmentBorderBoxIterator,
) {
    fn doit(
        flow: &mut dyn Flow,
        level: i32,
        iterator: &mut dyn FragmentBorderBoxIterator,
        stacking_context_position: &Point2D<Au>,
    ) {
        flow.iterate_through_fragment_border_boxes(iterator, level, stacking_context_position);

        for kid in flow.mut_base().child_iter_mut() {
            let mut stacking_context_position = *stacking_context_position;
            if kid.is_block_flow() && kid.as_block().fragment.establishes_stacking_context() {
                stacking_context_position =
                    Point2D::new(kid.as_block().fragment.margin.inline_start, Au(0)) +
                        kid.base().stacking_relative_position +
                        stacking_context_position.to_vector();
                let relative_position = kid
                    .as_block()
                    .stacking_relative_border_box(CoordinateSystem::Own);
                if let Some(matrix) = kid.as_block().fragment.transform_matrix(&relative_position) {
                    let transform_matrix = matrix.transform_point2d(&LayoutPoint::zero()).unwrap();
                    stacking_context_position = stacking_context_position + Vector2D::new(
                        Au::from_f32_px(transform_matrix.x),
                        Au::from_f32_px(transform_matrix.y),
                    )
                }
            }
            doit(kid, level + 1, iterator, &stacking_context_position);
        }
    }

    doit(root, 0, iterator, &Point2D::zero());
}

pub fn store_overflow(layout_context: &LayoutContext, flow: &mut dyn Flow) {
    if !flow
        .base()
        .restyle_damage
        .contains(ServoRestyleDamage::STORE_OVERFLOW)
    {
        return;
    }

    for kid in flow.mut_base().child_iter_mut() {
        store_overflow(layout_context, kid);
    }

    flow.store_overflow(layout_context);

    flow.mut_base()
        .restyle_damage
        .remove(ServoRestyleDamage::STORE_OVERFLOW);
}
