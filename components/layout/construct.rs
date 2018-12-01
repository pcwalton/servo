/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Creates flows and fragments from a DOM tree via a bottom-up, incremental traversal of the DOM.
//!
//! Each step of the traversal considers the node and existing flow, if there is one. If a node is
//! not dirty and an existing flow exists, then the traversal reuses that flow. Otherwise, it
//! proceeds to construct either a flow or a `ConstructionItem`. A construction item is a piece of
//! intermediate data that goes with a DOM node and hasn't found its "home" yet-maybe it's a box,
//! maybe it's an absolute or fixed position thing that hasn't found its containing block yet.
//! Construction items bubble up the tree from children to parents until they find their homes.

use crate::context::LayoutContext;
use crate::data::{LayoutData, LayoutDataFlags};
use crate::fragment::{CanvasFragmentInfo, Fragment, IframeFragmentInfo};
use crate::fragment::{ImageFragmentInfo};
use crate::fragment::{MediaFragmentInfo, SpecificFragmentInfo, SvgFragmentInfo};
use crate::wrapper::{LayoutNodeLayoutData, ThreadSafeLayoutNodeHelpers};
use crate::ServoArc;
use script_layout_interface::wrapper_traits::{
    PseudoElementType, ThreadSafeLayoutElement, ThreadSafeLayoutNode,
};
use script_layout_interface::{is_image_data, LayoutElementType, LayoutNodeType};
use servo_config::opts;
use servo_url::ServoUrl;
use std::marker::PhantomData;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use style::computed_values::display::T as Display;
use style::computed_values::float::T as Float;
use style::computed_values::position::T as Position;
use style::context::SharedStyleContext;
use style::dom::{OpaqueNode, TElement};
use style::properties::ComputedValues;
use style::selector_parser::RestyleDamage;
use style::servo::restyle_damage::ServoRestyleDamage;

/// The results of flow construction for a DOM node.
#[derive(Clone)]
pub enum ConstructionResult {
    /// This node contributes nothing at all (`display: none`). Alternately, this is what newly
    /// created nodes have their `ConstructionResult` set to.
    None,

    /// This node contributed some object or objects that will be needed to construct a proper flow
    /// later up the tree, but these objects have not yet found their home.
    ConstructionItem(ConstructionItem),
}

impl ConstructionResult {
    pub fn get(&mut self) -> ConstructionResult {
        // FIXME(pcwalton): Stop doing this with inline fragments. Cloning fragments is very
        // inefficient!
        (*self).clone()
    }

    pub fn debug_id(&self) -> usize {
        match *self {
            ConstructionResult::None => 0,
            ConstructionResult::ConstructionItem(_) => 0,
        }
    }
}

/// Represents the output of flow construction for a DOM node that has not yet resulted in a
/// complete flow. Construction items bubble up the tree until they find a `Flow` to be attached
/// to.
#[derive(Clone)]
pub enum ConstructionItem {
    /// Potentially ignorable whitespace.
    ///
    /// FIXME(emilio): How could whitespace have any PseudoElementType other
    /// than Normal?
    Whitespace(
        OpaqueNode,
        PseudoElementType,
        ServoArc<ComputedValues>,
        RestyleDamage,
    ),
}

/// Holds inline fragments that we're gathering for children of an inline node.
/// An object that knows how to create flows.
pub struct FlowConstructor<'a, N: ThreadSafeLayoutNode> {
    /// The layout context.
    pub layout_context: &'a LayoutContext<'a>,
    /// Satisfy the compiler about the unused parameters, which we use to improve the ergonomics of
    /// the ensuing impl {} by removing the need to parameterize all the methods individually.
    phantom2: PhantomData<N>,
}

impl<'a, ConcreteThreadSafeLayoutNode: ThreadSafeLayoutNode>
    FlowConstructor<'a, ConcreteThreadSafeLayoutNode>
{
    /// Creates a new flow constructor.
    pub fn new(layout_context: &'a LayoutContext<'a>) -> Self {
        FlowConstructor {
            layout_context: layout_context,
            phantom2: PhantomData,
        }
    }

    #[inline]
    fn style_context(&self) -> &SharedStyleContext {
        self.layout_context.shared_context()
    }

    #[inline]
    fn set_flow_construction_result(
        &self,
        node: &ConcreteThreadSafeLayoutNode,
        result: ConstructionResult,
    ) {
        node.set_flow_construction_result(result);
    }

    /// Builds the fragment for the given block or subclass thereof.
    fn build_fragment_for_block(&self, node: &ConcreteThreadSafeLayoutNode) -> Fragment {
        let specific_fragment_info = match node.type_id() {
            Some(LayoutNodeType::Element(LayoutElementType::HTMLIFrameElement)) => {
                SpecificFragmentInfo::Iframe(IframeFragmentInfo::new(node))
            },
            Some(LayoutNodeType::Element(LayoutElementType::HTMLImageElement)) => {
                let image_info = Box::new(ImageFragmentInfo::new(
                    node.image_url(),
                    node.image_density(),
                    node,
                    &self.layout_context,
                ));
                SpecificFragmentInfo::Image(image_info)
            },
            Some(LayoutNodeType::Element(LayoutElementType::HTMLMediaElement)) => {
                let data = node.media_data().unwrap();
                SpecificFragmentInfo::Media(Box::new(MediaFragmentInfo::new(data)))
            },
            Some(LayoutNodeType::Element(LayoutElementType::HTMLObjectElement)) => {
                let image_info = Box::new(ImageFragmentInfo::new(
                    node.object_data(),
                    None,
                    node,
                    &self.layout_context,
                ));
                SpecificFragmentInfo::Image(image_info)
            },
            Some(LayoutNodeType::Element(LayoutElementType::HTMLCanvasElement)) => {
                let data = node.canvas_data().unwrap();
                SpecificFragmentInfo::Canvas(Box::new(CanvasFragmentInfo::new(data)))
            },
            Some(LayoutNodeType::Element(LayoutElementType::SVGSVGElement)) => {
                let data = node.svg_data().unwrap();
                SpecificFragmentInfo::Svg(Box::new(SvgFragmentInfo::new(data)))
            },
            _ => {
                // This includes pseudo-elements.
                SpecificFragmentInfo::Generic
            },
        };

        Fragment::new(node, specific_fragment_info, self.layout_context)
    }

    fn build_block_flow_using_construction_result_of_child(
        &mut self,
        _: ConcreteThreadSafeLayoutNode,
    ) {
    }

    /// Constructs a block flow, beginning with the given `initial_fragments` if present and then
    /// appending the construction results of children to the child list of the block flow. {ib}
    /// splits and absolutely-positioned descendants are handled correctly.
    fn build_flow_for_block_starting_with_fragments(
        &mut self,
        node: &ConcreteThreadSafeLayoutNode,
    ) -> ConstructionResult {
        // List of absolute descendants, in tree order.
        if !node.is_replaced_content() {
            for kid in node.children() {
                self.build_block_flow_using_construction_result_of_child(kid);
            }
        }
        ConstructionResult::None
    }

    /// Constructs a flow for the given block node and its children. This method creates an
    /// initial fragment as appropriate and then dispatches to
    /// `build_flow_for_block_starting_with_fragments`. Currently the following kinds of flows get
    /// initial content:
    ///
    /// * Generated content gets the initial content specified by the `content` attribute of the
    ///   CSS.
    /// * `<input>` and `<textarea>` elements get their content.
    ///
    /// FIXME(pcwalton): It is not clear to me that there isn't a cleaner way to handle
    /// `<textarea>`.
    fn build_flow_for_block_like(&mut self, node: &ConcreteThreadSafeLayoutNode)
                                 -> ConstructionResult {
        let node_is_input_or_text_area =
            node.type_id() == Some(LayoutNodeType::Element(LayoutElementType::HTMLInputElement)) ||
                node.type_id() == Some(LayoutNodeType::Element(
                    LayoutElementType::HTMLTextAreaElement,
                ));
        if node.get_pseudo_element_type().is_replaced_content() || node_is_input_or_text_area {
            // A TextArea's text contents are displayed through the input text
            // box, so don't construct them.
            if node.type_id() == Some(LayoutNodeType::Element(
                LayoutElementType::HTMLTextAreaElement,
            )) {
                for kid in node.children() {
                    self.set_flow_construction_result(&kid, ConstructionResult::None)
                }
            }
        }
        self.build_flow_for_block_starting_with_fragments(node)
    }

    /// Builds a flow for a node with `display: block`. This yields a `BlockFlow` with possibly
    /// other `BlockFlow`s or `InlineFlow`s underneath it, depending on whether {ib} splits needed
    /// to happen.
    fn build_flow_for_block(&mut self, node: &ConcreteThreadSafeLayoutNode) -> ConstructionResult {
        let fragment = self.build_fragment_for_block(node);
        self.build_flow_for_block_like(node)
    }

    /// Attempts to perform incremental repair to account for recent changes to this node. This
    /// can fail and return false, indicating that flows will need to be reconstructed.
    ///
    /// TODO(pcwalton): Add some more fast paths, like toggling `display: none`, adding block kids
    /// to block parents with no {ib} splits, adding out-of-flow kids, etc.
    pub fn repair_if_possible(&mut self, node: &ConcreteThreadSafeLayoutNode) -> bool {
        // We can skip reconstructing the flow if we don't have to reconstruct and none of our kids
        // did either.
        //
        // We visit the kids first and reset their HAS_NEWLY_CONSTRUCTED_FLOW flags after checking
        // them.  NOTE: Make sure not to bail out early before resetting all the flags!
        let mut need_to_reconstruct = false;

        // If the node has display: none, it's possible that we haven't even
        // styled the children once, so we need to bailout early here.
        if node.style(self.style_context()).get_box().clone_display() == Display::None {
            return false;
        }

        for kid in node.children() {
            if kid
                .flags()
                .contains(LayoutDataFlags::HAS_NEWLY_CONSTRUCTED_FLOW)
            {
                kid.remove_flags(LayoutDataFlags::HAS_NEWLY_CONSTRUCTED_FLOW);
                need_to_reconstruct = true
            }
        }

        if need_to_reconstruct {
            return false;
        }

        if node
            .restyle_damage()
            .contains(ServoRestyleDamage::RECONSTRUCT_FLOW)
        {
            return false;
        }

        let set_has_newly_constructed_flow_flag = false;
        let result = {
            let style = node.style(self.style_context());

            if style.can_be_fragmented() || style.is_multicol() {
                return false;
            }

            let damage = node.restyle_damage();
            let mut data = node.mutate_layout_data().unwrap();

            match *node.construction_result_mut(&mut *data) {
                ConstructionResult::None => true,
                ConstructionResult::ConstructionItem(_) => false,
            }
        };
        if set_has_newly_constructed_flow_flag {
            node.insert_flags(LayoutDataFlags::HAS_NEWLY_CONSTRUCTED_FLOW);
        }
        return result;
    }
}

/// A utility trait with some useful methods for node queries.
trait NodeUtils {
    /// Returns true if this node doesn't render its kids and false otherwise.
    fn is_replaced_content(&self) -> bool;

    fn construction_result_mut(self, layout_data: &mut LayoutData) -> &mut ConstructionResult;

    /// Sets the construction result of a flow.
    fn set_flow_construction_result(self, result: ConstructionResult);

    /// Returns the construction result for this node.
    fn get_construction_result(self) -> ConstructionResult;
}

impl<ConcreteThreadSafeLayoutNode> NodeUtils for ConcreteThreadSafeLayoutNode
where
    ConcreteThreadSafeLayoutNode: ThreadSafeLayoutNode,
{
    fn is_replaced_content(&self) -> bool {
        match self.type_id() {
            Some(LayoutNodeType::Text) |
            Some(LayoutNodeType::Element(LayoutElementType::HTMLImageElement)) |
            Some(LayoutNodeType::Element(LayoutElementType::HTMLMediaElement)) |
            Some(LayoutNodeType::Element(LayoutElementType::HTMLIFrameElement)) |
            Some(LayoutNodeType::Element(LayoutElementType::HTMLCanvasElement)) |
            Some(LayoutNodeType::Element(LayoutElementType::SVGSVGElement)) => true,
            Some(LayoutNodeType::Element(LayoutElementType::HTMLObjectElement)) => {
                self.has_object_data()
            },
            Some(LayoutNodeType::Element(_)) => false,
            None => self.get_pseudo_element_type().is_replaced_content(),
        }
    }

    fn construction_result_mut(self, data: &mut LayoutData) -> &mut ConstructionResult {
        match self.get_pseudo_element_type() {
            PseudoElementType::Before => &mut data.before_flow_construction_result,
            PseudoElementType::After => &mut data.after_flow_construction_result,
            PseudoElementType::DetailsSummary => &mut data.details_summary_flow_construction_result,
            PseudoElementType::DetailsContent => &mut data.details_content_flow_construction_result,
            PseudoElementType::Normal => &mut data.flow_construction_result,
        }
    }

    #[inline(always)]
    fn set_flow_construction_result(self, result: ConstructionResult) {
        let mut layout_data = self.mutate_layout_data().unwrap();
        let dst = self.construction_result_mut(&mut *layout_data);
        *dst = result;
    }

    #[inline(always)]
    fn get_construction_result(self) -> ConstructionResult {
        let mut layout_data = self.mutate_layout_data().unwrap();
        self.construction_result_mut(&mut *layout_data).get()
    }
}

/// Methods for interacting with HTMLObjectElement nodes
trait ObjectElement {
    /// Returns true if this node has object data that is correct uri.
    fn has_object_data(&self) -> bool;

    /// Returns the "data" attribute value parsed as a URL
    fn object_data(&self) -> Option<ServoUrl>;
}

impl<N> ObjectElement for N
where
    N: ThreadSafeLayoutNode,
{
    fn has_object_data(&self) -> bool {
        let elem = self.as_element().unwrap();
        let type_and_data = (
            elem.get_attr(&ns!(), &local_name!("type")),
            elem.get_attr(&ns!(), &local_name!("data")),
        );
        match type_and_data {
            (None, Some(uri)) => is_image_data(uri),
            _ => false,
        }
    }

    fn object_data(&self) -> Option<ServoUrl> {
        let elem = self.as_element().unwrap();
        let type_and_data = (
            elem.get_attr(&ns!(), &local_name!("type")),
            elem.get_attr(&ns!(), &local_name!("data")),
        );
        match type_and_data {
            (None, Some(uri)) if is_image_data(uri) => ServoUrl::parse(uri).ok(),
            _ => None,
        }
    }
}

/// Convenience methods for computed CSS values
trait ComputedValueUtils {
    /// Returns true if this node has non-zero padding or border.
    fn has_padding_or_border(&self) -> bool;
}

impl ComputedValueUtils for ComputedValues {
    fn has_padding_or_border(&self) -> bool {
        let padding = self.get_padding();
        let border = self.get_border();

        !padding.padding_top.is_definitely_zero() ||
            !padding.padding_right.is_definitely_zero() ||
            !padding.padding_bottom.is_definitely_zero() ||
            !padding.padding_left.is_definitely_zero() ||
            border.border_top_width.px() != 0. ||
            border.border_right_width.px() != 0. ||
            border.border_bottom_width.px() != 0. ||
            border.border_left_width.px() != 0.
    }
}
