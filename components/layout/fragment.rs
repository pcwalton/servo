/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! The `Fragment` type, which represents the leaves of the layout tree.

use app_units::Au;
use canvas_traits::canvas::{CanvasId, CanvasMsg};
use crate::context::LayoutContext;
#[cfg(debug_assertions)]
use crate::layout_debug;
use crate::model::style_length;
use crate::model::{self, IntrinsicISizes, IntrinsicISizesContribution, MaybeAuto, SizeConstraint};
use crate::wrapper::ThreadSafeLayoutNodeHelpers;
use crate::ServoArc;
use euclid::{Point2D, Rect, Size2D, Vector2D};
use gfx::text::glyph::ByteIndex;
use gfx::text::text_run::{TextRun, TextRunSlice};
use gfx_traits::StackingContextId;
use ipc_channel::ipc::IpcSender;
use msg::constellation_msg::{BrowsingContextId, PipelineId};
use net_traits::image::base::{Image, ImageMetadata};
use net_traits::image_cache::{ImageOrMetadataAvailable, UsePlaceholder};
use range::*;
use script_layout_interface::wrapper_traits::{PseudoElementType, ThreadSafeLayoutNode};
use script_layout_interface::{HTMLCanvasData, HTMLCanvasDataSource, HTMLMediaData, SVGSVGData};
use serde::ser::{Serialize, SerializeStruct, Serializer};
use servo_url::ServoUrl;
use std::borrow::ToOwned;
use std::cmp::{max, min, Ordering};
use std::sync::{Arc, Mutex};
use std::fmt;
use style::computed_values::border_collapse::T as BorderCollapse;
use style::computed_values::box_sizing::T as BoxSizing;
use style::computed_values::color::T as Color;
use style::computed_values::display::T as Display;
use style::computed_values::mix_blend_mode::T as MixBlendMode;
use style::computed_values::overflow_wrap::T as OverflowWrap;
use style::computed_values::position::T as Position;
use style::computed_values::text_decoration_line::T as TextDecorationLine;
use style::computed_values::transform_style::T as TransformStyle;
use style::computed_values::white_space::T as WhiteSpace;
use style::computed_values::word_break::T as WordBreak;
use style::dom::OpaqueNode;
use style::logical_geometry::{Direction, LogicalMargin, LogicalRect, LogicalSize, WritingMode};
use style::properties::ComputedValues;
use style::selector_parser::RestyleDamage;
use style::servo::restyle_damage::ServoRestyleDamage;
use style::str::char_is_whitespace;
use style::values::computed::LengthOrPercentageOrAuto;
use style::values::generics::box_::Perspective;
use webrender_api::{self, LayoutTransform};

// From gfxFontConstants.h in Firefox.
// https://drafts.csswg.org/css-images/#default-object-size
static DEFAULT_REPLACED_WIDTH: i32 = 300;
static DEFAULT_REPLACED_HEIGHT: i32 = 150;

/// Fragments (`struct Fragment`) are the leaves of the layout tree. They cannot position
/// themselves. In general, fragments do not have a simple correspondence with CSS fragments in the
/// specification:
///
/// * Several fragments may correspond to the same CSS box or DOM node. For example, a CSS text box
/// broken across two lines is represented by two fragments.
///
/// * Some CSS fragments are not created at all, such as some anonymous block fragments induced by
///   inline fragments with block-level sibling fragments. In that case, Servo uses an `InlineFlow`
///   with `BlockFlow` siblings; the `InlineFlow` is block-level, but not a block container. It is
///   positioned as if it were a block fragment, but its children are positioned according to
///   inline flow.
///
/// A `SpecificFragmentInfo::Generic` is an empty fragment that contributes only borders, margins,
/// padding, and backgrounds. It is analogous to a CSS nonreplaced content box.
///
/// A fragment's type influences how its styles are interpreted during layout. For example,
/// replaced content such as images are resized differently from tables, text, or other content.
/// Different types of fragments may also contain custom data; for example, text fragments contain
/// text.
///
/// Do not add fields to this structure unless they're really really mega necessary! Fragments get
/// moved around a lot and thus their size impacts performance of layout quite a bit.
///
/// FIXME(#2260, pcwalton): This can be slimmed down some by (at least) moving `inline_context`
/// to be on `InlineFlow` only.
#[derive(Clone)]
pub struct Fragment {
    /// An opaque reference to the DOM node that this `Fragment` originates from.
    pub node: OpaqueNode,

    /// The CSS style of this fragment.
    pub style: ServoArc<ComputedValues>,

    /// The CSS style of this fragment when it's selected
    pub selected_style: ServoArc<ComputedValues>,

    /// The position of this fragment relative to its owning flow. The size includes padding and
    /// border, but not margin.
    ///
    /// NB: This does not account for relative positioning.
    /// NB: Collapsed borders are not included in this.
    pub border_box: LogicalRect<Au>,

    /// The sum of border and padding; i.e. the distance from the edge of the border box to the
    /// content edge of the fragment.
    pub border_padding: LogicalMargin<Au>,

    /// The margin of the content box.
    pub margin: LogicalMargin<Au>,

    /// Info specific to the kind of fragment. Keep this enum small.
    pub specific: SpecificFragmentInfo,

    /// How damaged this fragment is since last reflow.
    pub restyle_damage: RestyleDamage,

    /// The pseudo-element that this fragment represents.
    pub pseudo: PseudoElementType,

    /// Various flags for this fragment.
    pub flags: FragmentFlags,

    /// A debug ID that is consistent for the life of this fragment (via transform etc).
    /// This ID should not be considered stable across multiple layouts or fragment
    /// manipulations.
    debug_id: DebugId,

    /// The ID of the StackingContext that contains this fragment. This is initialized
    /// to 0, but it assigned during the collect_stacking_contexts phase of display
    /// list construction.
    pub stacking_context_id: StackingContextId,
}

impl Serialize for Fragment {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut serializer = serializer.serialize_struct("fragment", 3)?;
        serializer.serialize_field("id", &self.debug_id)?;
        serializer.serialize_field("border_box", &self.border_box)?;
        serializer.serialize_field("margin", &self.margin)?;
        serializer.end()
    }
}

/// Info specific to the kind of fragment.
///
/// Keep this enum small. As in, no more than one word. Or pcwalton will yell at you.
#[derive(Clone)]
pub enum SpecificFragmentInfo {
    Generic,

    Iframe(IframeFragmentInfo),
    Image(Box<ImageFragmentInfo>),
    Media(Box<MediaFragmentInfo>),
    Canvas(Box<CanvasFragmentInfo>),
    Svg(Box<SvgFragmentInfo>),

    ScannedText(Box<ScannedTextFragmentInfo>),
    UnscannedText(Box<UnscannedTextFragmentInfo>),

    /// A container for a fragment that got truncated by text-overflow.
    /// "Totally truncated fragments" are not rendered at all.
    /// Text fragments may be partially truncated (in which case this renders like a text fragment).
    /// Other fragments can only be totally truncated or not truncated at all.
    TruncatedFragment(Box<TruncatedFragmentInfo>),
}

impl SpecificFragmentInfo {
    fn restyle_damage(&self) -> RestyleDamage {
        RestyleDamage::empty()
    }

    pub fn get_type(&self) -> &'static str {
        match *self {
            SpecificFragmentInfo::Canvas(_) => "SpecificFragmentInfo::Canvas",
            SpecificFragmentInfo::Media(_) => "SpecificFragmentInfo::Media",
            SpecificFragmentInfo::Generic => "SpecificFragmentInfo::Generic",
            SpecificFragmentInfo::Iframe(_) => "SpecificFragmentInfo::Iframe",
            SpecificFragmentInfo::Image(_) => "SpecificFragmentInfo::Image",
            SpecificFragmentInfo::ScannedText(_) => "SpecificFragmentInfo::ScannedText",
            SpecificFragmentInfo::Svg(_) => "SpecificFragmentInfo::Svg",
            SpecificFragmentInfo::UnscannedText(_) => "SpecificFragmentInfo::UnscannedText",
            SpecificFragmentInfo::TruncatedFragment(_) => "SpecificFragmentInfo::TruncatedFragment",
        }
    }
}

impl fmt::Debug for SpecificFragmentInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            SpecificFragmentInfo::ScannedText(ref info) => write!(f, "{:?}", info.text()),
            SpecificFragmentInfo::UnscannedText(ref info) => write!(f, "{:?}", info.text),
            _ => Ok(()),
        }
    }
}

#[derive(Clone)]
pub enum CanvasFragmentSource {
    WebGL(webrender_api::ImageKey),
    Image(Option<Arc<Mutex<IpcSender<CanvasMsg>>>>),
}

#[derive(Clone)]
pub struct CanvasFragmentInfo {
    pub source: CanvasFragmentSource,
    pub dom_width: Au,
    pub dom_height: Au,
    pub canvas_id: CanvasId,
}

impl CanvasFragmentInfo {
    pub fn new(data: HTMLCanvasData) -> CanvasFragmentInfo {
        let source = match data.source {
            HTMLCanvasDataSource::WebGL(texture_id) => CanvasFragmentSource::WebGL(texture_id),
            HTMLCanvasDataSource::Image(ipc_sender) => CanvasFragmentSource::Image(
                ipc_sender.map(|renderer| Arc::new(Mutex::new(renderer))),
            ),
        };

        CanvasFragmentInfo {
            source: source,
            dom_width: Au::from_px(data.width as i32),
            dom_height: Au::from_px(data.height as i32),
            canvas_id: data.canvas_id,
        }
    }
}

#[derive(Clone)]
pub struct MediaFragmentInfo {
    pub current_frame: Option<(webrender_api::ImageKey, i32, i32)>,
}

impl MediaFragmentInfo {
    pub fn new(data: HTMLMediaData) -> MediaFragmentInfo {
        MediaFragmentInfo {
            current_frame: data.current_frame,
        }
    }
}

#[derive(Clone)]
pub struct SvgFragmentInfo {
    pub dom_width: Au,
    pub dom_height: Au,
}

impl SvgFragmentInfo {
    pub fn new(data: SVGSVGData) -> SvgFragmentInfo {
        SvgFragmentInfo {
            dom_width: Au::from_px(data.width as i32),
            dom_height: Au::from_px(data.height as i32),
        }
    }
}

/// A fragment that represents a replaced content image and its accompanying borders, shadows, etc.
#[derive(Clone)]
pub struct ImageFragmentInfo {
    pub image: Option<Arc<Image>>,
    pub metadata: Option<ImageMetadata>,
}

enum ImageOrMetadata {
    Image(Arc<Image>),
    Metadata(ImageMetadata),
}

impl ImageFragmentInfo {
    /// Creates a new image fragment from the given URL and local image cache.
    ///
    /// FIXME(pcwalton): The fact that image fragments store the cache in the fragment makes little
    /// sense to me.
    pub fn new<N: ThreadSafeLayoutNode>(
        url: Option<ServoUrl>,
        density: Option<f64>,
        node: &N,
        layout_context: &LayoutContext,
    ) -> ImageFragmentInfo {
        // First use any image data present in the element...
        let image_or_metadata = node
            .image_data()
            .and_then(|(image, metadata)| match (image, metadata) {
                (Some(image), _) => Some(ImageOrMetadata::Image(image)),
                (None, Some(metadata)) => Some(ImageOrMetadata::Metadata(metadata)),
                _ => None,
            })
            .or_else(|| {
                url.and_then(|url| {
                    // Otherwise query the image cache for anything known about the associated source URL.
                    layout_context
                        .get_or_request_image_or_meta(node.opaque(), url, UsePlaceholder::Yes)
                        .map(|result| match result {
                            ImageOrMetadataAvailable::ImageAvailable(i, _) => {
                                ImageOrMetadata::Image(i)
                            },
                            ImageOrMetadataAvailable::MetadataAvailable(m) => {
                                ImageOrMetadata::Metadata(m)
                            },
                        })
                })
            });

        let current_pixel_density = density.unwrap_or(1f64);

        let (image, metadata) = match image_or_metadata {
            Some(ImageOrMetadata::Image(i)) => {
                let height = (i.height as f64 / current_pixel_density) as u32;
                let width = (i.width as f64 / current_pixel_density) as u32;
                (
                    Some(Arc::new(Image {
                        height: height,
                        width: width,
                        ..(*i).clone()
                    })),
                    Some(ImageMetadata {
                        height: height,
                        width: width,
                    }),
                )
            },
            Some(ImageOrMetadata::Metadata(m)) => (
                None,
                Some(ImageMetadata {
                    height: (m.height as f64 / current_pixel_density) as u32,
                    width: (m.width as f64 / current_pixel_density) as u32,
                }),
            ),
            None => (None, None),
        };

        ImageFragmentInfo {
            image: image,
            metadata: metadata,
        }
    }
}

/// A fragment that represents an inline frame (iframe). This stores the frame ID so that the
/// size of this iframe can be communicated via the constellation to the iframe's own layout thread.
#[derive(Clone)]
pub struct IframeFragmentInfo {
    /// The frame ID of this iframe. None if there is no nested browsing context.
    pub browsing_context_id: Option<BrowsingContextId>,
    /// The pipelineID of this iframe. None if there is no nested browsing context.
    pub pipeline_id: Option<PipelineId>,
}

impl IframeFragmentInfo {
    /// Creates the information specific to an iframe fragment.
    pub fn new<N: ThreadSafeLayoutNode>(node: &N) -> IframeFragmentInfo {
        let browsing_context_id = node.iframe_browsing_context_id();
        let pipeline_id = node.iframe_pipeline_id();
        IframeFragmentInfo {
            browsing_context_id: browsing_context_id,
            pipeline_id: pipeline_id,
        }
    }
}

/// A scanned text fragment represents a single run of text with a distinct style. A `TextFragment`
/// may be split into two or more fragments across line breaks. Several `TextFragment`s may
/// correspond to a single DOM text node. Split text fragments are implemented by referring to
/// subsets of a single `TextRun` object.
#[derive(Clone)]
pub struct ScannedTextFragmentInfo {
    /// The text run that this represents.
    pub run: Arc<TextRun>,

    /// The intrinsic size of the text fragment.
    pub content_size: LogicalSize<Au>,

    /// The byte offset of the insertion point, if any.
    pub insertion_point: Option<ByteIndex>,

    /// The range within the above text run that this represents.
    pub range: Range<ByteIndex>,

    /// The endpoint of the above range, including whitespace that was stripped out. This exists
    /// so that we can restore the range to its original value (before line breaking occurred) when
    /// performing incremental reflow.
    pub range_end_including_stripped_whitespace: ByteIndex,

    pub flags: ScannedTextFlags,
}

bitflags! {
    pub struct ScannedTextFlags: u8 {
        /// Whether a line break is required after this fragment if wrapping on newlines (e.g. if
        /// `white-space: pre` is in effect).
        const REQUIRES_LINE_BREAK_AFTERWARD_IF_WRAPPING_ON_NEWLINES = 0x01;

        /// Is this fragment selected?
        const SELECTED = 0x02;

        /// Suppress line breaking between this and the previous fragment
        ///
        /// This handles cases like Foo<span>bar</span>
        const SUPPRESS_LINE_BREAK_BEFORE = 0x04;
    }
}

impl ScannedTextFragmentInfo {
    /// Creates the information specific to a scanned text fragment from a range and a text run.
    pub fn new(
        run: Arc<TextRun>,
        range: Range<ByteIndex>,
        content_size: LogicalSize<Au>,
        insertion_point: Option<ByteIndex>,
        flags: ScannedTextFlags,
    ) -> ScannedTextFragmentInfo {
        ScannedTextFragmentInfo {
            run: run,
            range: range,
            insertion_point: insertion_point,
            content_size: content_size,
            range_end_including_stripped_whitespace: range.end(),
            flags: flags,
        }
    }

    pub fn text(&self) -> &str {
        &self.run.text[self.range.begin().to_usize()..self.range.end().to_usize()]
    }

    pub fn requires_line_break_afterward_if_wrapping_on_newlines(&self) -> bool {
        self.flags
            .contains(ScannedTextFlags::REQUIRES_LINE_BREAK_AFTERWARD_IF_WRAPPING_ON_NEWLINES)
    }

    pub fn selected(&self) -> bool {
        self.flags.contains(ScannedTextFlags::SELECTED)
    }
}

/// Describes how to split a fragment. This is used during line breaking as part of the return
/// value of `find_split_info_for_inline_size()`.
#[derive(Clone, Debug)]
pub struct SplitInfo {
    // TODO(bjz): this should only need to be a single character index, but both values are
    // currently needed for splitting in the `inline::try_append_*` functions.
    pub range: Range<ByteIndex>,
    pub inline_size: Au,
}

impl SplitInfo {
    fn new(range: Range<ByteIndex>, info: &ScannedTextFragmentInfo) -> SplitInfo {
        let inline_size = info.run.advance_for_range(&range);
        SplitInfo {
            range: range,
            inline_size: inline_size,
        }
    }
}

/// Describes how to split a fragment into two. This contains up to two `SplitInfo`s.
pub struct SplitResult {
    /// The part of the fragment that goes on the first line.
    pub inline_start: Option<SplitInfo>,
    /// The part of the fragment that goes on the second line.
    pub inline_end: Option<SplitInfo>,
    /// The text run which is being split.
    pub text_run: Arc<TextRun>,
}

/// Describes how a fragment should be truncated.
struct TruncationResult {
    /// The part of the fragment remaining after truncation.
    split: SplitInfo,
    /// The text run which is being truncated.
    text_run: Arc<TextRun>,
}

/// Data for an unscanned text fragment. Unscanned text fragments are the results of flow
/// construction that have not yet had their inline-size determined.
#[derive(Clone)]
pub struct UnscannedTextFragmentInfo {
    /// The text inside the fragment.
    pub text: Box<str>,

    /// The selected text range.  An empty range represents the insertion point.
    pub selection: Option<Range<ByteIndex>>,
}

impl UnscannedTextFragmentInfo {
    /// Creates a new instance of `UnscannedTextFragmentInfo` from the given text.
    #[inline]
    pub fn new(text: Box<str>, selection: Option<Range<ByteIndex>>) -> UnscannedTextFragmentInfo {
        UnscannedTextFragmentInfo {
            text: text,
            selection: selection,
        }
    }
}

/// A wrapper for fragments that have been truncated by the `text-overflow` property.
/// This may have an associated text node, or, if the fragment was completely truncated,
/// it may act as an invisible marker for incremental reflow.
#[derive(Clone)]
pub struct TruncatedFragmentInfo {
    pub text_info: Option<ScannedTextFragmentInfo>,
    pub full: Fragment,
}

impl Fragment {
    /// Constructs a new `Fragment` instance.
    pub fn new<N: ThreadSafeLayoutNode>(
        node: &N,
        specific: SpecificFragmentInfo,
        ctx: &LayoutContext,
    ) -> Fragment {
        let shared_context = ctx.shared_context();
        let style = node.style(shared_context);
        let writing_mode = style.writing_mode;

        let mut restyle_damage = node.restyle_damage();
        restyle_damage.remove(ServoRestyleDamage::RECONSTRUCT_FLOW);

        Fragment {
            node: node.opaque(),
            style: style,
            selected_style: node.selected_style(),
            restyle_damage: restyle_damage,
            border_box: LogicalRect::zero(writing_mode),
            border_padding: LogicalMargin::zero(writing_mode),
            margin: LogicalMargin::zero(writing_mode),
            specific: specific,
            pseudo: node.get_pseudo_element_type(),
            flags: FragmentFlags::empty(),
            debug_id: DebugId::new(),
            stacking_context_id: StackingContextId::root(),
        }
    }

    /// Constructs a new `Fragment` instance from an opaque node.
    pub fn from_opaque_node_and_style(
        node: OpaqueNode,
        pseudo: PseudoElementType,
        style: ServoArc<ComputedValues>,
        selected_style: ServoArc<ComputedValues>,
        mut restyle_damage: RestyleDamage,
        specific: SpecificFragmentInfo,
    ) -> Fragment {
        let writing_mode = style.writing_mode;

        restyle_damage.remove(ServoRestyleDamage::RECONSTRUCT_FLOW);

        Fragment {
            node: node,
            style: style,
            selected_style: selected_style,
            restyle_damage: restyle_damage,
            border_box: LogicalRect::zero(writing_mode),
            border_padding: LogicalMargin::zero(writing_mode),
            margin: LogicalMargin::zero(writing_mode),
            specific: specific,
            pseudo: pseudo,
            flags: FragmentFlags::empty(),
            debug_id: DebugId::new(),
            stacking_context_id: StackingContextId::root(),
        }
    }

    /// Creates an anonymous fragment just like this one but with the given style and fragment
    /// type. For the new anonymous fragment, layout-related values (border box, etc.) are reset to
    /// initial values.
    pub fn create_similar_anonymous_fragment(
        &self,
        style: ServoArc<ComputedValues>,
        specific: SpecificFragmentInfo,
    ) -> Fragment {
        let writing_mode = style.writing_mode;
        Fragment {
            node: self.node,
            style: style,
            selected_style: self.selected_style.clone(),
            restyle_damage: self.restyle_damage,
            border_box: LogicalRect::zero(writing_mode),
            border_padding: LogicalMargin::zero(writing_mode),
            margin: LogicalMargin::zero(writing_mode),
            specific: specific,
            pseudo: self.pseudo,
            flags: FragmentFlags::empty(),
            debug_id: DebugId::new(),
            stacking_context_id: StackingContextId::root(),
        }
    }

    /// Transforms this fragment into another fragment of the given type, with the given size,
    /// preserving all the other data.
    pub fn transform(&self, size: LogicalSize<Au>, info: SpecificFragmentInfo) -> Fragment {
        let new_border_box =
            LogicalRect::from_point_size(self.style.writing_mode, self.border_box.start, size);

        let mut restyle_damage = RestyleDamage::rebuild_and_reflow();
        restyle_damage.remove(ServoRestyleDamage::RECONSTRUCT_FLOW);

        Fragment {
            node: self.node,
            style: self.style.clone(),
            selected_style: self.selected_style.clone(),
            restyle_damage: restyle_damage,
            border_box: new_border_box,
            border_padding: self.border_padding,
            margin: self.margin,
            specific: info,
            pseudo: self.pseudo.clone(),
            flags: FragmentFlags::empty(),
            debug_id: self.debug_id.clone(),
            stacking_context_id: StackingContextId::root(),
        }
    }

    /// Transforms this fragment using the given `SplitInfo`, preserving all the other data.
    ///
    /// If this is the first half of a split, `first` is true
    pub fn transform_with_split_info(
        &self,
        split: &SplitInfo,
        text_run: Arc<TextRun>,
        first: bool,
    ) -> Fragment {
        let size = LogicalSize::new(
            self.style.writing_mode,
            split.inline_size,
            self.border_box.size.block,
        );
        // Preserve the insertion point if it is in this fragment's range or it is at line end.
        let (mut flags, insertion_point) = match self.specific {
            SpecificFragmentInfo::ScannedText(ref info) => match info.insertion_point {
                Some(index) if split.range.contains(index) => (info.flags, info.insertion_point),
                Some(index)
                    if index == ByteIndex(text_run.text.chars().count() as isize - 1) &&
                        index == split.range.end() =>
                {
                    (info.flags, info.insertion_point)
                },
                _ => (info.flags, None),
            },
            _ => (ScannedTextFlags::empty(), None),
        };

        if !first {
            flags.set(ScannedTextFlags::SUPPRESS_LINE_BREAK_BEFORE, false);
        }

        let info = Box::new(ScannedTextFragmentInfo::new(
            text_run,
            split.range,
            size,
            insertion_point,
            flags,
        ));
        self.transform(size, SpecificFragmentInfo::ScannedText(info))
    }

    pub fn restyle_damage(&self) -> RestyleDamage {
        self.restyle_damage | self.specific.restyle_damage()
    }

    pub fn contains_node(&self, node_address: OpaqueNode) -> bool {
        node_address == self.node
    }

    /// Determines which quantities (border/padding/margin/specified) should be included in the
    /// intrinsic inline size of this fragment.
    fn quantities_included_in_intrinsic_inline_size(
        &self,
    ) -> QuantitiesIncludedInIntrinsicInlineSizes {
        match self.specific {
            SpecificFragmentInfo::Canvas(_) |
            SpecificFragmentInfo::Media(_) |
            SpecificFragmentInfo::Generic |
            SpecificFragmentInfo::Iframe(_) |
            SpecificFragmentInfo::Image(_) |
            SpecificFragmentInfo::Svg(_) => {
                QuantitiesIncludedInIntrinsicInlineSizes::all()
            }
            SpecificFragmentInfo::TruncatedFragment(_) |
            SpecificFragmentInfo::ScannedText(_) |
            SpecificFragmentInfo::UnscannedText(_) => {
                QuantitiesIncludedInIntrinsicInlineSizes::empty()
            }
        }
    }

    /// Returns the portion of the intrinsic inline-size that consists of borders/padding and
    /// margins, respectively.
    ///
    /// FIXME(#2261, pcwalton): This won't work well for inlines: is this OK?
    pub fn surrounding_intrinsic_inline_size(&self) -> (Au, Au) {
        let flags = self.quantities_included_in_intrinsic_inline_size();
        let style = self.style();

        // FIXME(pcwalton): Percentages should be relative to any definite size per CSS-SIZING.
        // This will likely need to be done by pushing down definite sizes during selector
        // cascading.
        let margin = if flags.contains(
            QuantitiesIncludedInIntrinsicInlineSizes::INTRINSIC_INLINE_SIZE_INCLUDES_MARGINS,
        ) {
            let margin = style.logical_margin();
            (MaybeAuto::from_style(margin.inline_start, Au(0)).specified_or_zero() +
                MaybeAuto::from_style(margin.inline_end, Au(0)).specified_or_zero())
        } else {
            Au(0)
        };

        // FIXME(pcwalton): Percentages should be relative to any definite size per CSS-SIZING.
        // This will likely need to be done by pushing down definite sizes during selector
        // cascading.
        let padding = if flags.contains(
            QuantitiesIncludedInIntrinsicInlineSizes::INTRINSIC_INLINE_SIZE_INCLUDES_PADDING,
        ) {
            let padding = style.logical_padding();
            (padding.inline_start.to_used_value(Au(0)) + padding.inline_end.to_used_value(Au(0)))
        } else {
            Au(0)
        };

        let border = if flags.contains(
            QuantitiesIncludedInIntrinsicInlineSizes::INTRINSIC_INLINE_SIZE_INCLUDES_BORDER,
        ) {
            self.border_width().inline_start_end()
        } else {
            Au(0)
        };

        (border + padding, margin)
    }

    /// Uses the style only to estimate the intrinsic inline-sizes. These may be modified for text
    /// or replaced elements.
    pub fn style_specified_intrinsic_inline_size(&self) -> IntrinsicISizesContribution {
        let flags = self.quantities_included_in_intrinsic_inline_size();
        let style = self.style();

        // FIXME(#2261, pcwalton): This won't work well for inlines: is this OK?
        let (border_padding, margin) = self.surrounding_intrinsic_inline_size();

        let mut specified = Au(0);
        if flags.contains(
            QuantitiesIncludedInIntrinsicInlineSizes::INTRINSIC_INLINE_SIZE_INCLUDES_SPECIFIED,
        ) {
            specified =
                MaybeAuto::from_style(style.content_inline_size(), Au(0)).specified_or_zero();
            specified = max(style.min_inline_size().to_used_value(Au(0)), specified);
            if let Some(max) = style.max_inline_size().to_used_value(Au(0)) {
                specified = min(specified, max)
            }

            if self.style.get_position().box_sizing == BoxSizing::BorderBox {
                specified = max(Au(0), specified - border_padding);
            }
        }

        IntrinsicISizesContribution {
            content_intrinsic_sizes: IntrinsicISizes {
                minimum_inline_size: specified,
                preferred_inline_size: specified,
            },
            surrounding_size: border_padding + margin,
        }
    }

    /// intrinsic width of this replaced element.
    #[inline]
    pub fn intrinsic_width(&self) -> Au {
        match self.specific {
            SpecificFragmentInfo::Image(ref info) => {
                if let Some(ref data) = info.metadata {
                    Au::from_px(data.width as i32)
                } else {
                    Au(0)
                }
            },
            SpecificFragmentInfo::Media(ref info) => {
                if let Some((_, width, _)) = info.current_frame {
                    Au::from_px(width as i32)
                } else {
                    Au(0)
                }
            },
            SpecificFragmentInfo::Canvas(ref info) => info.dom_width,
            SpecificFragmentInfo::Svg(ref info) => info.dom_width,
            // Note: Currently for replaced element with no intrinsic size,
            // this function simply returns the default object size. As long as
            // these elements do not have intrinsic aspect ratio this should be
            // sufficient, but we may need to investigate if this is enough for
            // use cases like SVG.
            SpecificFragmentInfo::Iframe(_) => Au::from_px(DEFAULT_REPLACED_WIDTH),
            _ => panic!("Trying to get intrinsic width on non-replaced element!"),
        }
    }

    /// intrinsic width of this replaced element.
    #[inline]
    pub fn intrinsic_height(&self) -> Au {
        match self.specific {
            SpecificFragmentInfo::Image(ref info) => {
                if let Some(ref data) = info.metadata {
                    Au::from_px(data.height as i32)
                } else {
                    Au(0)
                }
            },
            SpecificFragmentInfo::Media(ref info) => {
                if let Some((_, _, height)) = info.current_frame {
                    Au::from_px(height as i32)
                } else {
                    Au(0)
                }
            },
            SpecificFragmentInfo::Canvas(ref info) => info.dom_height,
            SpecificFragmentInfo::Svg(ref info) => info.dom_height,
            SpecificFragmentInfo::Iframe(_) => Au::from_px(DEFAULT_REPLACED_HEIGHT),
            _ => panic!("Trying to get intrinsic height on non-replaced element!"),
        }
    }

    /// Whether this replace element has intrinsic aspect ratio.
    pub fn has_intrinsic_ratio(&self) -> bool {
        match self.specific {
            SpecificFragmentInfo::Image(_)  |
            SpecificFragmentInfo::Canvas(_) |
            SpecificFragmentInfo::Media(_) |
            // TODO(stshine): According to the SVG spec, whether a SVG element has intrinsic
            // aspect ratio is determined by the `preserveAspectRatio` attribute. Since for
            // now SVG is far from implemented, we simply choose the default behavior that
            // the intrinsic aspect ratio is preserved.
            // https://svgwg.org/svg2-draft/coords.html#PreserveAspectRatioAttribute
            SpecificFragmentInfo::Svg(_) =>
                self.intrinsic_width() != Au(0) && self.intrinsic_height() != Au(0),
            _ => false
        }
    }

    /// CSS 2.1 § 10.3.2 & 10.6.2 Calculate the used width and height of a replaced element.
    /// When a parameter is `None` it means the specified size in certain direction
    /// is unconstrained. The inline containing size can also be `None` since this
    /// method is also used for calculating intrinsic inline size contribution.
    pub fn calculate_replaced_sizes(
        &self,
        containing_inline_size: Option<Au>,
        containing_block_size: Option<Au>,
    ) -> (Au, Au) {
        let (intrinsic_inline_size, intrinsic_block_size) = if self.style.writing_mode.is_vertical()
        {
            (self.intrinsic_height(), self.intrinsic_width())
        } else {
            (self.intrinsic_width(), self.intrinsic_height())
        };

        // Make sure the size we used here is for content box since they may be
        // transferred by the intrinsic aspect ratio.
        let inline_size = style_length(self.style.content_inline_size(), containing_inline_size)
            .map(|x| x - self.box_sizing_boundary(Direction::Inline));
        let block_size = style_length(self.style.content_block_size(), containing_block_size)
            .map(|x| x - self.box_sizing_boundary(Direction::Block));
        let inline_constraint = self.size_constraint(containing_inline_size, Direction::Inline);
        let block_constraint = self.size_constraint(containing_block_size, Direction::Block);

        // https://drafts.csswg.org/css-images-3/#default-sizing
        match (inline_size, block_size) {
            // If the specified size is a definite width and height, the concrete
            // object size is given that width and height.
            (MaybeAuto::Specified(inline_size), MaybeAuto::Specified(block_size)) => (
                inline_constraint.clamp(inline_size),
                block_constraint.clamp(block_size),
            ),

            // If the specified size is only a width or height (but not both)
            // then the concrete object size is given that specified width or
            // height. The other dimension is calculated as follows:
            //
            // If the object has an intrinsic aspect ratio, the missing dimension
            // of the concrete object size is calculated using the intrinsic
            // aspect ratio and the present dimension.
            //
            // Otherwise, if the missing dimension is present in the object’s intrinsic
            // dimensions, the missing dimension is taken from the object’s intrinsic
            // dimensions. Otherwise it is taken from the default object size.
            (MaybeAuto::Specified(inline_size), MaybeAuto::Auto) => {
                let inline_size = inline_constraint.clamp(inline_size);
                let block_size = if self.has_intrinsic_ratio() {
                    // Note: We can not precompute the ratio and store it as a float, because
                    // doing so may result one pixel difference in calculation for certain
                    // images, thus make some tests fail.
                    Au::new(
                        (inline_size.0 as i64 * intrinsic_block_size.0 as i64 /
                            intrinsic_inline_size.0 as i64) as i32,
                    )
                } else {
                    intrinsic_block_size
                };
                (inline_size, block_constraint.clamp(block_size))
            },
            (MaybeAuto::Auto, MaybeAuto::Specified(block_size)) => {
                let block_size = block_constraint.clamp(block_size);
                let inline_size = if self.has_intrinsic_ratio() {
                    Au::new(
                        (block_size.0 as i64 * intrinsic_inline_size.0 as i64 /
                            intrinsic_block_size.0 as i64) as i32,
                    )
                } else {
                    intrinsic_inline_size
                };
                (inline_constraint.clamp(inline_size), block_size)
            },
            // https://drafts.csswg.org/css2/visudet.html#min-max-widths
            (MaybeAuto::Auto, MaybeAuto::Auto) => {
                if self.has_intrinsic_ratio() {
                    // This approch follows the spirit of cover and contain constraint.
                    // https://drafts.csswg.org/css-images-3/#cover-contain

                    // First, create two rectangles that keep aspect ratio while may be clamped
                    // by the contraints;
                    let first_isize = inline_constraint.clamp(intrinsic_inline_size);
                    let first_bsize = Au::new(
                        (first_isize.0 as i64 * intrinsic_block_size.0 as i64 /
                            intrinsic_inline_size.0 as i64) as i32,
                    );
                    let second_bsize = block_constraint.clamp(intrinsic_block_size);
                    let second_isize = Au::new(
                        (second_bsize.0 as i64 * intrinsic_inline_size.0 as i64 /
                            intrinsic_block_size.0 as i64) as i32,
                    );
                    let (inline_size, block_size) = match (
                        first_isize.cmp(&intrinsic_inline_size),
                        second_isize.cmp(&intrinsic_inline_size),
                    ) {
                        (Ordering::Equal, Ordering::Equal) => (first_isize, first_bsize),
                        // When only one rectangle is clamped, use it;
                        (Ordering::Equal, _) => (second_isize, second_bsize),
                        (_, Ordering::Equal) => (first_isize, first_bsize),
                        // When both rectangles grow (smaller than min sizes),
                        // Choose the larger one;
                        (Ordering::Greater, Ordering::Greater) => {
                            if first_isize > second_isize {
                                (first_isize, first_bsize)
                            } else {
                                (second_isize, second_bsize)
                            }
                        },
                        // When both rectangles shrink (larger than max sizes),
                        // Choose the smaller one;
                        (Ordering::Less, Ordering::Less) => {
                            if first_isize > second_isize {
                                (second_isize, second_bsize)
                            } else {
                                (first_isize, first_bsize)
                            }
                        },
                        // It does not matter which we choose here, because both sizes
                        // will be clamped to constraint;
                        (Ordering::Less, Ordering::Greater) |
                        (Ordering::Greater, Ordering::Less) => (first_isize, first_bsize),
                    };
                    // Clamp the result and we are done :-)
                    (
                        inline_constraint.clamp(inline_size),
                        block_constraint.clamp(block_size),
                    )
                } else {
                    (
                        inline_constraint.clamp(intrinsic_inline_size),
                        block_constraint.clamp(intrinsic_block_size),
                    )
                }
            },
        }
    }

    /// Return a size constraint that can be used the clamp size in given direction.
    /// To take `box-sizing: border-box` into account, the `border_padding` field
    /// must be initialized first.
    ///
    /// TODO(stshine): Maybe there is a more convenient way.
    pub fn size_constraint(
        &self,
        containing_size: Option<Au>,
        direction: Direction,
    ) -> SizeConstraint {
        let (style_min_size, style_max_size) = match direction {
            Direction::Inline => (self.style.min_inline_size(), self.style.max_inline_size()),
            Direction::Block => (self.style.min_block_size(), self.style.max_block_size()),
        };

        let border = if self.style().get_position().box_sizing == BoxSizing::BorderBox {
            Some(self.border_padding.start_end(direction))
        } else {
            None
        };

        SizeConstraint::new(containing_size, style_min_size, style_max_size, border)
    }

    /// Returns a guess as to the distances from the margin edge of this fragment to its content
    /// in the inline direction. This will generally be correct unless percentages are involved.
    ///
    /// This is used for the float placement speculation logic.
    pub fn guess_inline_content_edge_offsets(&self) -> SpeculatedInlineContentEdgeOffsets {
        let logical_margin = self.style.logical_margin();
        let logical_padding = self.style.logical_padding();
        let border_width = self.border_width();
        SpeculatedInlineContentEdgeOffsets {
            start: MaybeAuto::from_style(logical_margin.inline_start, Au(0)).specified_or_zero() +
                logical_padding.inline_start.to_used_value(Au(0)) +
                border_width.inline_start,
            end: MaybeAuto::from_style(logical_margin.inline_end, Au(0)).specified_or_zero() +
                logical_padding.inline_end.to_used_value(Au(0)) +
                border_width.inline_end,
        }
    }

    /// Returns the sum of the inline-sizes of all the borders of this fragment. Note that this
    /// can be expensive to compute, so if possible use the `border_padding` field instead.
    #[inline]
    pub fn border_width(&self) -> LogicalMargin<Au> {
        let style_border_width = self.style().logical_border_width();

        // NOTE: We can have nodes with different writing mode inside
        // the inline fragment context, so we need to overwrite the
        // writing mode to compute the child logical sizes.
        let writing_mode = self.style.writing_mode;
        let context_border = LogicalMargin::zero(writing_mode);
        style_border_width + context_border
    }

    /// Returns the border width in given direction if this fragment has property
    /// 'box-sizing: border-box'. The `border_padding` field must have been initialized.
    pub fn box_sizing_boundary(&self, direction: Direction) -> Au {
        match (self.style().get_position().box_sizing, direction) {
            (BoxSizing::BorderBox, Direction::Inline) => self.border_padding.inline_start_end(),
            (BoxSizing::BorderBox, Direction::Block) => self.border_padding.block_start_end(),
            _ => Au(0),
        }
    }

    /// Computes the margins in the inline direction from the containing block inline-size and the
    /// style. After this call, the inline direction of the `margin` field will be correct.
    ///
    /// Do not use this method if the inline direction margins are to be computed some other way
    /// (for example, via constraint solving for blocks).
    pub fn compute_inline_direction_margins(&mut self, containing_block_inline_size: Au) {
        match self.specific {
            _ => {
                let margin = self.style().logical_margin();
                self.margin.inline_start =
                    MaybeAuto::from_style(margin.inline_start, containing_block_inline_size)
                        .specified_or_zero();
                self.margin.inline_end =
                    MaybeAuto::from_style(margin.inline_end, containing_block_inline_size)
                        .specified_or_zero();
            },
        }
    }

    /// Computes the margins in the block direction from the containing block inline-size and the
    /// style. After this call, the block direction of the `margin` field will be correct.
    ///
    /// Do not use this method if the block direction margins are to be computed some other way
    /// (for example, via constraint solving for absolutely-positioned flows).
    pub fn compute_block_direction_margins(&mut self, containing_block_inline_size: Au) {
        // NB: Percentages are relative to containing block inline-size (not block-size)
        // per CSS 2.1.
        let margin = self.style().logical_margin();
        self.margin.block_start =
            MaybeAuto::from_style(margin.block_start, containing_block_inline_size)
                .specified_or_zero();
        self.margin.block_end =
            MaybeAuto::from_style(margin.block_end, containing_block_inline_size)
                .specified_or_zero();
    }

    /// Computes the border and padding in both inline and block directions from the containing
    /// block inline-size and the style. After this call, the `border_padding` field will be
    /// correct.
    pub fn compute_border_and_padding(&mut self, containing_block_inline_size: Au) {
        // Compute border.
        let border = match self.style.get_inherited_table().border_collapse {
            BorderCollapse::Separate => self.border_width(),
            BorderCollapse::Collapse => LogicalMargin::zero(self.style.writing_mode),
        };

        // Compute padding from the fragment's style.
        let padding_from_style = model::padding_from_style(
            self.style(),
            containing_block_inline_size,
            self.style().writing_mode,
        );

        self.border_padding = border + padding_from_style
    }

    // Return offset from original position because of `position: relative`.
    pub fn relative_position(&self, containing_block_size: &LogicalSize<Au>) -> LogicalSize<Au> {
        fn from_style(style: &ComputedValues, container_size: &LogicalSize<Au>) -> LogicalSize<Au> {
            let offsets = style.logical_position();
            let offset_i = if offsets.inline_start != LengthOrPercentageOrAuto::Auto {
                MaybeAuto::from_style(offsets.inline_start, container_size.inline)
                    .specified_or_zero()
            } else {
                -MaybeAuto::from_style(offsets.inline_end, container_size.inline)
                    .specified_or_zero()
            };
            let offset_b = if offsets.block_start != LengthOrPercentageOrAuto::Auto {
                MaybeAuto::from_style(offsets.block_start, container_size.block).specified_or_zero()
            } else {
                -MaybeAuto::from_style(offsets.block_end, container_size.block).specified_or_zero()
            };
            LogicalSize::new(style.writing_mode, offset_i, offset_b)
        }

        // Go over the ancestor fragments and add all relative offsets (if any).
        let rel_pos = if self.style().get_box().position == Position::Relative {
            from_style(self.style(), containing_block_size)
        } else {
            LogicalSize::zero(self.style.writing_mode)
        };

        rel_pos
    }

    #[inline(always)]
    pub fn style(&self) -> &ComputedValues {
        &*self.style
    }

    #[inline(always)]
    pub fn selected_style(&self) -> &ComputedValues {
        &*self.selected_style
    }

    pub fn white_space(&self) -> WhiteSpace {
        self.style().get_inherited_text().white_space
    }

    pub fn color(&self) -> Color {
        self.style().get_color().color
    }

    /// Returns the text decoration line of this fragment, according to the style of the nearest ancestor
    /// element.
    ///
    /// NB: This may not be the actual text decoration line, because of the override rules specified in
    /// CSS 2.1 § 16.3.1. Unfortunately, computing this properly doesn't really fit into Servo's
    /// model. Therefore, this is a best lower bound approximation, but the end result may actually
    /// have the various decoration flags turned on afterward.
    pub fn text_decoration_line(&self) -> TextDecorationLine {
        self.style().get_text().text_decoration_line
    }

    /// Returns the inline-start offset from margin edge to content edge.
    ///
    /// FIXME(#2262, pcwalton): I think this method is pretty bogus, because it won't work for
    /// inlines.
    pub fn inline_start_offset(&self) -> Au {
        self.margin.inline_start + self.border_padding.inline_start
    }

    /// Returns true if this element can be split. This is true for text fragments, unless
    /// `white-space: pre` or `white-space: nowrap` is set.
    pub fn can_split(&self) -> bool {
        self.is_scanned_text_fragment() && self.white_space().allow_wrap()
    }

    /// Returns true if and only if this is a scanned text fragment.
    pub fn is_scanned_text_fragment(&self) -> bool {
        match self.specific {
            SpecificFragmentInfo::ScannedText(..) => true,
            _ => false,
        }
    }

    pub fn suppress_line_break_before(&self) -> bool {
        match self.specific {
            SpecificFragmentInfo::ScannedText(ref st) => st
                .flags
                .contains(ScannedTextFlags::SUPPRESS_LINE_BREAK_BEFORE),
            _ => false,
        }
    }

    /// Computes the intrinsic inline-sizes of this fragment.
    pub fn compute_intrinsic_inline_sizes(&mut self) -> IntrinsicISizesContribution {
        let mut result = self.style_specified_intrinsic_inline_size();
        match self.specific {
            SpecificFragmentInfo::Generic => {}
            SpecificFragmentInfo::Image(_) |
            SpecificFragmentInfo::Media(_) |
            SpecificFragmentInfo::Canvas(_) |
            SpecificFragmentInfo::Iframe(_) |
            SpecificFragmentInfo::Svg(_) => {
                let mut inline_size = match self.style.content_inline_size() {
                    LengthOrPercentageOrAuto::Auto | LengthOrPercentageOrAuto::Percentage(_) => {
                        // We have to initialize the `border_padding` field first to make
                        // the size constraints work properly.
                        // TODO(stshine): Find a cleaner way to do this.
                        let padding = self.style.logical_padding();
                        self.border_padding.inline_start =
                            padding.inline_start.to_used_value(Au(0));
                        self.border_padding.inline_end = padding.inline_end.to_used_value(Au(0));
                        self.border_padding.block_start = padding.block_start.to_used_value(Au(0));
                        self.border_padding.block_end = padding.block_end.to_used_value(Au(0));
                        let border = self.border_width();
                        self.border_padding.inline_start += border.inline_start;
                        self.border_padding.inline_end += border.inline_end;
                        self.border_padding.block_start += border.block_start;
                        self.border_padding.block_end += border.block_end;
                        let (result_inline, _) = self.calculate_replaced_sizes(None, None);
                        result_inline
                    },
                    LengthOrPercentageOrAuto::Length(length) => Au::from(length),
                    LengthOrPercentageOrAuto::Calc(calc) => {
                        // TODO(nox): This is probably wrong, because it accounts neither for
                        // clamping (not sure if necessary here) nor percentage.
                        Au::from(calc.unclamped_length())
                    },
                };

                let size_constraint = self.size_constraint(None, Direction::Inline);
                inline_size = size_constraint.clamp(inline_size);

                result.union_block(&IntrinsicISizes {
                    minimum_inline_size: inline_size,
                    preferred_inline_size: inline_size,
                });
            },

            SpecificFragmentInfo::TruncatedFragment(ref t) if t.text_info.is_some() => {
                let text_fragment_info = t.text_info.as_ref().unwrap();
                handle_text(text_fragment_info, self, &mut result)
            },
            SpecificFragmentInfo::ScannedText(ref text_fragment_info) => {
                handle_text(text_fragment_info, self, &mut result)
            },

            SpecificFragmentInfo::TruncatedFragment(_) => return IntrinsicISizesContribution::new(),

            SpecificFragmentInfo::UnscannedText(..) => {
                panic!("Unscanned text fragments should have been scanned by now!")
            },
        };

        fn handle_text(
            text_fragment_info: &ScannedTextFragmentInfo,
            self_: &Fragment,
            result: &mut IntrinsicISizesContribution,
        ) {
            let range = &text_fragment_info.range;

            // See http://dev.w3.org/csswg/css-sizing/#max-content-inline-size.
            // TODO: Account for soft wrap opportunities.
            let max_line_inline_size = text_fragment_info
                .run
                .metrics_for_range(range)
                .advance_width;

            let min_line_inline_size = if self_.white_space().allow_wrap() {
                text_fragment_info.run.min_width_for_range(range)
            } else {
                max_line_inline_size
            };

            result.union_block(&IntrinsicISizes {
                minimum_inline_size: min_line_inline_size,
                preferred_inline_size: max_line_inline_size,
            })
        }

        result
    }

    /// Returns the narrowest inline-size that the first splittable part of this fragment could
    /// possibly be split to. (In most cases, this returns the inline-size of the first word in
    /// this fragment.)
    pub fn minimum_splittable_inline_size(&self) -> Au {
        match self.specific {
            SpecificFragmentInfo::TruncatedFragment(ref t) if t.text_info.is_some() => {
                let text = t.text_info.as_ref().unwrap();
                text.run.minimum_splittable_inline_size(&text.range)
            },
            SpecificFragmentInfo::ScannedText(ref text) => {
                text.run.minimum_splittable_inline_size(&text.range)
            },
            _ => Au(0),
        }
    }

    /// Returns the dimensions of the content box.
    ///
    /// This is marked `#[inline]` because it is frequently called when only one or two of the
    /// values are needed and that will save computation.
    #[inline]
    pub fn content_box(&self) -> LogicalRect<Au> {
        self.border_box - self.border_padding
    }

    /// Attempts to find the split positions of a text fragment so that its inline-size is no more
    /// than `max_inline_size`.
    ///
    /// A return value of `None` indicates that the fragment could not be split. Otherwise the
    /// information pertaining to the split is returned. The inline-start and inline-end split
    /// information are both optional due to the possibility of them being whitespace.
    pub fn calculate_split_position(
        &self,
        max_inline_size: Au,
        starts_line: bool,
    ) -> Option<SplitResult> {
        let text_fragment_info = match self.specific {
            SpecificFragmentInfo::ScannedText(ref text_fragment_info) => text_fragment_info,
            _ => return None,
        };

        let mut flags = SplitOptions::empty();
        if starts_line {
            flags.insert(SplitOptions::STARTS_LINE);
            if self.style().get_inherited_text().overflow_wrap == OverflowWrap::BreakWord {
                flags.insert(SplitOptions::RETRY_AT_CHARACTER_BOUNDARIES)
            }
        }

        match self.style().get_inherited_text().word_break {
            WordBreak::Normal | WordBreak::KeepAll => {
                // Break at normal word boundaries. keep-all forbids soft wrap opportunities.
                let natural_word_breaking_strategy = text_fragment_info
                    .run
                    .natural_word_slices_in_range(&text_fragment_info.range);
                self.calculate_split_position_using_breaking_strategy(
                    natural_word_breaking_strategy,
                    max_inline_size,
                    flags,
                )
            },
            WordBreak::BreakAll => {
                // Break at character boundaries.
                let character_breaking_strategy = text_fragment_info
                    .run
                    .character_slices_in_range(&text_fragment_info.range);
                flags.remove(SplitOptions::RETRY_AT_CHARACTER_BOUNDARIES);
                self.calculate_split_position_using_breaking_strategy(
                    character_breaking_strategy,
                    max_inline_size,
                    flags,
                )
            },
        }
    }

    /// Does this fragment start on a glyph run boundary?
    pub fn is_on_glyph_run_boundary(&self) -> bool {
        let text_fragment_info = match self.specific {
            SpecificFragmentInfo::ScannedText(ref text_fragment_info) => text_fragment_info,
            _ => return true,
        };
        text_fragment_info
            .run
            .on_glyph_run_boundary(text_fragment_info.range.begin())
    }

    /// Truncates this fragment to the given `max_inline_size`, using a character-based breaking
    /// strategy. The resulting fragment will have `SpecificFragmentInfo::TruncatedFragment`,
    /// preserving the original fragment for use in incremental reflow.
    ///
    /// This function will panic if self is already truncated.
    pub fn truncate_to_inline_size(self, max_inline_size: Au) -> Fragment {
        if let SpecificFragmentInfo::TruncatedFragment(_) = self.specific {
            panic!("Cannot truncate an already truncated fragment");
        }
        let info = self.calculate_truncate_to_inline_size(max_inline_size);
        let (size, text_info) = match info {
            Some(TruncationResult {
                split: SplitInfo { inline_size, range },
                text_run,
            }) => {
                let size = LogicalSize::new(
                    self.style.writing_mode,
                    inline_size,
                    self.border_box.size.block,
                );
                // Preserve the insertion point if it is in this fragment's range or it is at line end.
                let (flags, insertion_point) = match self.specific {
                    SpecificFragmentInfo::ScannedText(ref info) => match info.insertion_point {
                        Some(index) if range.contains(index) => (info.flags, info.insertion_point),
                        Some(index)
                            if index == ByteIndex(text_run.text.chars().count() as isize - 1) &&
                                index == range.end() =>
                        {
                            (info.flags, info.insertion_point)
                        },
                        _ => (info.flags, None),
                    },
                    _ => (ScannedTextFlags::empty(), None),
                };
                let text_info =
                    ScannedTextFragmentInfo::new(text_run, range, size, insertion_point, flags);
                (size, Some(text_info))
            },
            None => (LogicalSize::zero(self.style.writing_mode), None),
        };
        let mut result = self.transform(size, SpecificFragmentInfo::Generic);
        result.specific =
            SpecificFragmentInfo::TruncatedFragment(Box::new(TruncatedFragmentInfo {
                text_info: text_info,
                full: self,
            }));
        result
    }

    /// Truncates this fragment to the given `max_inline_size`, using a character-based breaking
    /// strategy. If no characters could fit, returns `None`.
    fn calculate_truncate_to_inline_size(&self, max_inline_size: Au) -> Option<TruncationResult> {
        let text_fragment_info =
            if let SpecificFragmentInfo::ScannedText(ref text_fragment_info) = self.specific {
                text_fragment_info
            } else {
                return None;
            };

        let character_breaking_strategy = text_fragment_info
            .run
            .character_slices_in_range(&text_fragment_info.range);

        let split_info = self.calculate_split_position_using_breaking_strategy(
            character_breaking_strategy,
            max_inline_size,
            SplitOptions::empty(),
        )?;

        let split = split_info.inline_start?;
        Some(TruncationResult {
            split: split,
            text_run: split_info.text_run.clone(),
        })
    }

    /// A helper method that uses the breaking strategy described by `slice_iterator` (at present,
    /// either natural word breaking or character breaking) to split this fragment.
    fn calculate_split_position_using_breaking_strategy<'a, I>(
        &self,
        slice_iterator: I,
        max_inline_size: Au,
        flags: SplitOptions,
    ) -> Option<SplitResult>
    where
        I: Iterator<Item = TextRunSlice<'a>>,
    {
        let text_fragment_info = match self.specific {
            SpecificFragmentInfo::ScannedText(ref text_fragment_info) => text_fragment_info,
            _ => return None,
        };

        let mut remaining_inline_size = max_inline_size - self.border_padding.inline_start_end();
        let mut inline_start_range = Range::new(text_fragment_info.range.begin(), ByteIndex(0));
        let mut inline_end_range = None;
        let mut overflowing = false;

        debug!(
            "calculate_split_position_using_breaking_strategy: splitting text fragment \
             (strlen={}, range={:?}, max_inline_size={:?})",
            text_fragment_info.run.text.len(),
            text_fragment_info.range,
            max_inline_size
        );

        for slice in slice_iterator {
            debug!(
                "calculate_split_position_using_breaking_strategy: considering slice \
                 (offset={:?}, slice range={:?}, remaining_inline_size={:?})",
                slice.offset, slice.range, remaining_inline_size
            );

            // Use the `remaining_inline_size` to find a split point if possible. If not, go around
            // the loop again with the next slice.
            let metrics = text_fragment_info
                .run
                .metrics_for_slice(slice.glyphs, &slice.range);
            let advance = metrics.advance_width;

            // Have we found the split point?
            if advance <= remaining_inline_size || slice.glyphs.is_whitespace() {
                // Keep going; we haven't found the split point yet.
                debug!("calculate_split_position_using_breaking_strategy: enlarging span");
                remaining_inline_size = remaining_inline_size - advance;
                inline_start_range.extend_by(slice.range.length());
                continue;
            }

            // The advance is more than the remaining inline-size, so split here. First, check to
            // see if we're going to overflow the line. If so, perform a best-effort split.
            let mut remaining_range = slice.text_run_range();
            let split_is_empty = inline_start_range.is_empty() && !(self
                .requires_line_break_afterward_if_wrapping_on_newlines() &&
                !self.white_space().allow_wrap());
            if split_is_empty {
                // We're going to overflow the line.
                overflowing = true;
                inline_start_range = slice.text_run_range();
                remaining_range = Range::new(slice.text_run_range().end(), ByteIndex(0));
                remaining_range.extend_to(text_fragment_info.range.end());
            }

            // Check to see if we need to create an inline-end chunk.
            let slice_begin = remaining_range.begin();
            if slice_begin < text_fragment_info.range.end() {
                // There still some things left over at the end of the line, so create the
                // inline-end chunk.
                let mut inline_end = remaining_range;
                inline_end.extend_to(text_fragment_info.range.end());
                inline_end_range = Some(inline_end);
                debug!(
                    "calculate_split_position: splitting remainder with inline-end range={:?}",
                    inline_end
                );
            }

            // If we failed to find a suitable split point, we're on the verge of overflowing the
            // line.
            if split_is_empty || overflowing {
                // If we've been instructed to retry at character boundaries (probably via
                // `overflow-wrap: break-word`), do so.
                if flags.contains(SplitOptions::RETRY_AT_CHARACTER_BOUNDARIES) {
                    let character_breaking_strategy = text_fragment_info
                        .run
                        .character_slices_in_range(&text_fragment_info.range);
                    let mut flags = flags;
                    flags.remove(SplitOptions::RETRY_AT_CHARACTER_BOUNDARIES);
                    return self.calculate_split_position_using_breaking_strategy(
                        character_breaking_strategy,
                        max_inline_size,
                        flags,
                    );
                }

                // We aren't at the start of the line, so don't overflow. Let inline layout wrap to
                // the next line instead.
                if !flags.contains(SplitOptions::STARTS_LINE) {
                    return None;
                }
            }

            break;
        }

        let split_is_empty = inline_start_range.is_empty() &&
            !self.requires_line_break_afterward_if_wrapping_on_newlines();
        let inline_start = if !split_is_empty {
            Some(SplitInfo::new(inline_start_range, &**text_fragment_info))
        } else {
            None
        };
        let inline_end = inline_end_range
            .map(|inline_end_range| SplitInfo::new(inline_end_range, &**text_fragment_info));

        Some(SplitResult {
            inline_start: inline_start,
            inline_end: inline_end,
            text_run: text_fragment_info.run.clone(),
        })
    }

    /// Restore any whitespace that was stripped from a text fragment, and recompute inline metrics
    /// if necessary.
    pub fn reset_text_range_and_inline_size(&mut self) {
        if let SpecificFragmentInfo::ScannedText(ref mut info) = self.specific {
            if info.run.extra_word_spacing != Au(0) {
                Arc::make_mut(&mut info.run).extra_word_spacing = Au(0);
            }

            // FIXME (mbrubeck): Do we need to restore leading too?
            let range_end = info.range_end_including_stripped_whitespace;
            if info.range.end() == range_end {
                return;
            }
            info.range.extend_to(range_end);
            info.content_size.inline = info.run.metrics_for_range(&info.range).advance_width;
            self.border_box.size.inline =
                info.content_size.inline + self.border_padding.inline_start_end();
        }
    }

    /// Assigns replaced inline-size, padding, and margins for this fragment only if it is replaced
    /// content per CSS 2.1 § 10.3.2.
    pub fn assign_replaced_inline_size_if_necessary(
        &mut self,
        container_inline_size: Au,
        container_block_size: Option<Au>,
    ) {
        match self.specific {
            SpecificFragmentInfo::TruncatedFragment(ref t) if t.text_info.is_none() => return,
            SpecificFragmentInfo::Generic => return,
            SpecificFragmentInfo::UnscannedText(_) => {
                panic!("Unscanned text fragments should have been scanned by now!")
            },
            SpecificFragmentInfo::Canvas(_) |
            SpecificFragmentInfo::Image(_) |
            SpecificFragmentInfo::Media(_) |
            SpecificFragmentInfo::Iframe(_) |
            SpecificFragmentInfo::ScannedText(_) |
            SpecificFragmentInfo::TruncatedFragment(_) |
            SpecificFragmentInfo::Svg(_) => {},
        };

        match self.specific {
            // Text
            SpecificFragmentInfo::TruncatedFragment(ref t) if t.text_info.is_some() => {
                let info = t.text_info.as_ref().unwrap();
                // Scanned text fragments will have already had their content inline-sizes assigned
                // by this point.
                self.border_box.size.inline =
                    info.content_size.inline + self.border_padding.inline_start_end();
            },
            SpecificFragmentInfo::ScannedText(ref info) => {
                // Scanned text fragments will have already had their content inline-sizes assigned
                // by this point.
                self.border_box.size.inline =
                    info.content_size.inline + self.border_padding.inline_start_end();
            },

            // Replaced elements
            _ if self.is_replaced() => {
                let (inline_size, block_size) = self
                    .calculate_replaced_sizes(Some(container_inline_size), container_block_size);
                self.border_box.size.inline = inline_size + self.border_padding.inline_start_end();
                self.border_box.size.block = block_size + self.border_padding.block_start_end();
            },

            ref unhandled @ _ => {
                panic!("this case should have been handled above: {:?}", unhandled)
            },
        }
    }

    /// Assign block-size for this fragment if it is replaced content. The inline-size must have
    /// been assigned first.
    ///
    /// Ideally, this should follow CSS 2.1 § 10.6.2.
    pub fn assign_replaced_block_size_if_necessary(&mut self) {
        match self.specific {
            SpecificFragmentInfo::TruncatedFragment(ref t) if t.text_info.is_none() => return,
            SpecificFragmentInfo::Generic => return,
            SpecificFragmentInfo::UnscannedText(_) => {
                panic!("Unscanned text fragments should have been scanned by now!")
            },
            SpecificFragmentInfo::Canvas(_) |
            SpecificFragmentInfo::Iframe(_) |
            SpecificFragmentInfo::Image(_) |
            SpecificFragmentInfo::Media(_) |
            SpecificFragmentInfo::ScannedText(_) |
            SpecificFragmentInfo::TruncatedFragment(_) |
            SpecificFragmentInfo::Svg(_) => {},
        }

        match self.specific {
            // Text
            SpecificFragmentInfo::TruncatedFragment(ref t) if t.text_info.is_some() => {
                let info = t.text_info.as_ref().unwrap();
                // Scanned text fragments' content block-sizes are calculated by the text run
                // scanner during flow construction.
                self.border_box.size.block =
                    info.content_size.block + self.border_padding.block_start_end();
            },
            SpecificFragmentInfo::ScannedText(ref info) => {
                // Scanned text fragments' content block-sizes are calculated by the text run
                // scanner during flow construction.
                self.border_box.size.block =
                    info.content_size.block + self.border_padding.block_start_end();
            },

            // Replaced elements
            _ if self.is_replaced() => {},

            ref unhandled @ _ => panic!("should have been handled above: {:?}", unhandled),
        }
    }

    /// Returns true if this fragment is replaced content.
    pub fn is_replaced(&self) -> bool {
        match self.specific {
            SpecificFragmentInfo::Iframe(_) |
            SpecificFragmentInfo::Canvas(_) |
            SpecificFragmentInfo::Image(_) |
            SpecificFragmentInfo::Media(_) |
            SpecificFragmentInfo::Svg(_) => true,
            _ => false,
        }
    }

    /// Returns true if this fragment is replaced content or an inline-block or false otherwise.
    pub fn is_replaced_or_inline_block(&self) -> bool {
        self.is_replaced()
    }

    /// Returns true if this fragment is a hypothetical box. See CSS 2.1 § 10.3.7.
    pub fn is_hypothetical(&self) -> bool {
        false
    }

    /// Returns true if this fragment can merge with another immediately-following fragment or
    /// false otherwise.
    pub fn can_merge_with_fragment(&self, other: &Fragment) -> bool {
        match (&self.specific, &other.specific) {
            (
                &SpecificFragmentInfo::UnscannedText(ref first_unscanned_text),
                &SpecificFragmentInfo::UnscannedText(_),
            ) => {
                // FIXME: Should probably use a whitelist of styles that can safely differ (#3165)
                if self.style().get_font() != other.style().get_font() ||
                    self.text_decoration_line() != other.text_decoration_line() ||
                    self.white_space() != other.white_space() ||
                    self.color() != other.color()
                {
                    return false;
                }

                if first_unscanned_text.text.ends_with('\n') {
                    return false;
                }

                true
            },
            _ => false,
        }
    }

    /// Returns true if and only if this is the *primary fragment* for the fragment's style object
    /// (conceptually, though style sharing makes this not really true, of course). The primary
    /// fragment is the one that draws backgrounds, borders, etc., and takes borders, padding and
    /// margins into account. Every style object has at most one primary fragment.
    ///
    /// At present, all fragments are primary fragments except for inline-block and table wrapper
    /// fragments. Inline-block fragments are not primary fragments because the corresponding block
    /// flow is the primary fragment, while table wrapper fragments are not primary fragments
    /// because the corresponding table flow is the primary fragment.
    pub fn is_primary_fragment(&self) -> bool {
        match self.specific {
            SpecificFragmentInfo::Canvas(_) |
            SpecificFragmentInfo::Generic |
            SpecificFragmentInfo::Iframe(_) |
            SpecificFragmentInfo::Image(_) |
            SpecificFragmentInfo::Media(_) |
            SpecificFragmentInfo::ScannedText(_) |
            SpecificFragmentInfo::Svg(_) |
            SpecificFragmentInfo::TruncatedFragment(_) |
            SpecificFragmentInfo::UnscannedText(_) => true,
        }
    }

    /// Determines the inline sizes of inline-block fragments. These cannot be fully computed until
    /// inline size assignment has run for the child flow: thus it is computed "late", during
    /// block size assignment.
    pub fn update_late_computed_replaced_inline_size_if_necessary(&mut self) {
    }

    pub fn update_late_computed_inline_position_if_necessary(&mut self) {
    }

    pub fn update_late_computed_block_position_if_necessary(&mut self) {
    }

    pub fn repair_style(&mut self, new_style: &ServoArc<ComputedValues>) {
        self.style = (*new_style).clone()
    }

    /// Given the stacking-context-relative position of the containing flow, returns the border box
    /// of this fragment relative to the parent stacking context. This takes `position: relative`
    /// into account.
    ///
    /// If `coordinate_system` is `Parent`, this returns the border box in the parent stacking
    /// context's coordinate system. Otherwise, if `coordinate_system` is `Own` and this fragment
    /// establishes a stacking context itself, this returns a border box anchored at (0, 0). (If
    /// this fragment does not establish a stacking context, then it always belongs to its parent
    /// stacking context and thus `coordinate_system` is ignored.)
    ///
    /// This is the method you should use for display list construction as well as
    /// `getBoundingClientRect()` and so forth.
    pub fn stacking_relative_border_box(
        &self,
        stacking_relative_flow_origin: &Vector2D<Au>,
        relative_containing_block_size: &LogicalSize<Au>,
        relative_containing_block_mode: WritingMode,
        coordinate_system: CoordinateSystem,
    ) -> Rect<Au> {
        let container_size =
            relative_containing_block_size.to_physical(relative_containing_block_mode);
        let border_box = self
            .border_box
            .to_physical(self.style.writing_mode, container_size);
        if coordinate_system == CoordinateSystem::Own && self.establishes_stacking_context() {
            return Rect::new(Point2D::zero(), border_box.size);
        }

        // FIXME(pcwalton): This can double-count relative position sometimes for inlines (e.g.
        // `<div style="position:relative">x</div>`, because the `position:relative` trickles down
        // to the inline flow. Possibly we should extend the notion of "primary fragment" to fix
        // this.
        let relative_position = self.relative_position(relative_containing_block_size);
        border_box
            .translate_by_size(&relative_position.to_physical(self.style.writing_mode))
            .translate(&stacking_relative_flow_origin)
    }

    /// Given the stacking-context-relative border box, returns the stacking-context-relative
    /// content box.
    pub fn stacking_relative_content_box(
        &self,
        stacking_relative_border_box: Rect<Au>,
    ) -> Rect<Au> {
        let border_padding = self.border_padding.to_physical(self.style.writing_mode);
        Rect::new(
            Point2D::new(
                stacking_relative_border_box.origin.x + border_padding.left,
                stacking_relative_border_box.origin.y + border_padding.top,
            ),
            Size2D::new(
                stacking_relative_border_box.size.width - border_padding.horizontal(),
                stacking_relative_border_box.size.height - border_padding.vertical(),
            ),
        )
    }

    /// Returns true if this fragment may establish a reference frame.
    pub fn can_establish_reference_frame(&self) -> bool {
        !self.style().get_box().transform.0.is_empty() ||
            self.style().get_box().perspective != Perspective::None
    }

    /// Returns true if this fragment has a filter, transform, or perspective property set.
    pub fn has_filter_transform_or_perspective(&self) -> bool {
        !self.style().get_box().transform.0.is_empty() ||
            !self.style().get_effects().filter.0.is_empty() ||
            self.style().get_box().perspective != Perspective::None
    }

    /// Returns true if this fragment establishes a new stacking context and false otherwise.
    pub fn establishes_stacking_context(&self) -> bool {
        // Text fragments shouldn't create stacking contexts.
        match self.specific {
            SpecificFragmentInfo::TruncatedFragment(_) |
            SpecificFragmentInfo::ScannedText(_) |
            SpecificFragmentInfo::UnscannedText(_) => return false,
            _ => {},
        }

        if self.style().get_effects().opacity != 1.0 {
            return true;
        }

        if self.style().get_effects().mix_blend_mode != MixBlendMode::Normal {
            return true;
        }

        if self.has_filter_transform_or_perspective() {
            return true;
        }

        if self.style().get_box().transform_style == TransformStyle::Preserve3d ||
            self.style().overrides_transform_style()
        {
            return true;
        }

        // Fixed position and sticky position always create stacking contexts.
        if self.style().get_box().position == Position::Fixed ||
            self.style().get_box().position == Position::Sticky
        {
            return true;
        }

        // Statically positioned fragments don't establish stacking contexts if the previous
        // conditions are not fulfilled. Furthermore, z-index doesn't apply to statically
        // positioned fragments.
        if self.style().get_box().position == Position::Static {
            return false;
        }

        // For absolutely and relatively positioned fragments we only establish a stacking
        // context if there is a z-index set.
        // See https://www.w3.org/TR/CSS2/visuren.html#z-index
        !self.style().get_position().z_index.is_auto()
    }

    // Get the effective z-index of this fragment. Z-indices only apply to positioned element
    // per CSS 2 9.9.1 (http://www.w3.org/TR/CSS2/visuren.html#z-index), so this value may differ
    // from the value specified in the style.
    pub fn effective_z_index(&self) -> i32 {
        match self.style().get_box().position {
            Position::Static => {},
            _ => return self.style().get_position().z_index.integer_or(0),
        }

        if !self.style().get_box().transform.0.is_empty() {
            return self.style().get_position().z_index.integer_or(0);
        }

        match self.style().get_box().display {
            Display::Flex => self.style().get_position().z_index.integer_or(0),
            _ => 0,
        }
    }

    /// Computes the overflow rect of this fragment relative to the start of the flow.
    pub fn compute_overflow(
        &self,
        flow_size: &Size2D<Au>,
        relative_containing_block_size: &LogicalSize<Au>,
    ) -> Overflow {
        let mut border_box = self
            .border_box
            .to_physical(self.style.writing_mode, *flow_size);

        // Relative position can cause us to draw outside our border box.
        //
        // FIXME(pcwalton): I'm not a fan of the way this makes us crawl though so many styles all
        // the time. Can't we handle relative positioning by just adjusting `border_box`?
        let relative_position = self.relative_position(relative_containing_block_size);
        border_box =
            border_box.translate_by_size(&relative_position.to_physical(self.style.writing_mode));
        let mut overflow = Overflow::from_rect(&border_box);

        // Outlines cause us to draw outside our border box.
        let outline_width = Au::from(self.style.get_outline().outline_width);
        if outline_width != Au(0) {
            overflow.paint = overflow
                .paint
                .union(&border_box.inflate(outline_width, outline_width))
        }

        // FIXME(pcwalton): Sometimes excessively fancy glyphs can make us draw outside our border
        // box too.
        overflow
    }

    pub fn requires_line_break_afterward_if_wrapping_on_newlines(&self) -> bool {
        match self.specific {
            SpecificFragmentInfo::TruncatedFragment(ref t) if t.text_info.is_some() => {
                let text = t.text_info.as_ref().unwrap();
                text.requires_line_break_afterward_if_wrapping_on_newlines()
            },
            SpecificFragmentInfo::ScannedText(ref text) => {
                text.requires_line_break_afterward_if_wrapping_on_newlines()
            },
            _ => false,
        }
    }

    pub fn strip_leading_whitespace_if_necessary(&mut self) -> WhitespaceStrippingResult {
        if self.white_space().preserve_spaces() {
            return WhitespaceStrippingResult::RetainFragment;
        }

        return match self.specific {
            SpecificFragmentInfo::TruncatedFragment(ref mut t) if t.text_info.is_some() => {
                let scanned_text_fragment_info = t.text_info.as_mut().unwrap();
                scanned_text(scanned_text_fragment_info, &mut self.border_box)
            },
            SpecificFragmentInfo::ScannedText(ref mut scanned_text_fragment_info) => {
                scanned_text(scanned_text_fragment_info, &mut self.border_box)
            },
            SpecificFragmentInfo::UnscannedText(ref mut unscanned_text_fragment_info) => {
                let mut new_text_string = String::new();
                let mut modified = false;
                for (i, character) in unscanned_text_fragment_info.text.char_indices() {
                    if gfx::text::util::is_bidi_control(character) {
                        new_text_string.push(character);
                        continue;
                    }
                    if char_is_whitespace(character) {
                        modified = true;
                        continue;
                    }
                    // Finished processing leading control chars and whitespace.
                    if modified {
                        new_text_string.push_str(&unscanned_text_fragment_info.text[i..]);
                    }
                    break;
                }
                if modified {
                    unscanned_text_fragment_info.text = new_text_string.into_boxed_str();
                }

                WhitespaceStrippingResult::from_unscanned_text_fragment_info(
                    &unscanned_text_fragment_info,
                )
            },
            _ => WhitespaceStrippingResult::RetainFragment,
        };

        fn scanned_text(
            scanned_text_fragment_info: &mut ScannedTextFragmentInfo,
            border_box: &mut LogicalRect<Au>,
        ) -> WhitespaceStrippingResult {
            let leading_whitespace_byte_count = scanned_text_fragment_info
                .text()
                .find(|c| !char_is_whitespace(c))
                .unwrap_or(scanned_text_fragment_info.text().len());

            let whitespace_len = ByteIndex(leading_whitespace_byte_count as isize);
            let whitespace_range =
                Range::new(scanned_text_fragment_info.range.begin(), whitespace_len);
            let text_bounds = scanned_text_fragment_info
                .run
                .metrics_for_range(&whitespace_range)
                .bounding_box;
            border_box.size.inline = border_box.size.inline - text_bounds.size.width;
            scanned_text_fragment_info.content_size.inline =
                scanned_text_fragment_info.content_size.inline - text_bounds.size.width;

            scanned_text_fragment_info
                .range
                .adjust_by(whitespace_len, -whitespace_len);

            WhitespaceStrippingResult::RetainFragment
        }
    }

    /// Returns true if the entire fragment was stripped.
    pub fn strip_trailing_whitespace_if_necessary(&mut self) -> WhitespaceStrippingResult {
        if self.white_space().preserve_spaces() {
            return WhitespaceStrippingResult::RetainFragment;
        }

        return match self.specific {
            SpecificFragmentInfo::TruncatedFragment(ref mut t) if t.text_info.is_some() => {
                let scanned_text_fragment_info = t.text_info.as_mut().unwrap();
                scanned_text(scanned_text_fragment_info, &mut self.border_box)
            },
            SpecificFragmentInfo::ScannedText(ref mut scanned_text_fragment_info) => {
                scanned_text(scanned_text_fragment_info, &mut self.border_box)
            },
            SpecificFragmentInfo::UnscannedText(ref mut unscanned_text_fragment_info) => {
                let mut trailing_bidi_control_characters_to_retain = Vec::new();
                let (mut modified, mut last_character_index) = (true, 0);
                for (i, character) in unscanned_text_fragment_info.text.char_indices().rev() {
                    if gfx::text::util::is_bidi_control(character) {
                        trailing_bidi_control_characters_to_retain.push(character);
                        continue;
                    }
                    if char_is_whitespace(character) {
                        modified = true;
                        continue;
                    }
                    last_character_index = i + character.len_utf8();
                    break;
                }
                if modified {
                    let mut text = unscanned_text_fragment_info.text.to_string();
                    text.truncate(last_character_index);
                    for character in trailing_bidi_control_characters_to_retain.iter().rev() {
                        text.push(*character);
                    }
                    unscanned_text_fragment_info.text = text.into_boxed_str();
                }

                WhitespaceStrippingResult::from_unscanned_text_fragment_info(
                    &unscanned_text_fragment_info,
                )
            },
            _ => WhitespaceStrippingResult::RetainFragment,
        };

        fn scanned_text(
            scanned_text_fragment_info: &mut ScannedTextFragmentInfo,
            border_box: &mut LogicalRect<Au>,
        ) -> WhitespaceStrippingResult {
            let mut trailing_whitespace_start_byte = 0;
            for (i, c) in scanned_text_fragment_info.text().char_indices().rev() {
                if !char_is_whitespace(c) {
                    trailing_whitespace_start_byte = i + c.len_utf8();
                    break;
                }
            }
            let whitespace_start = ByteIndex(trailing_whitespace_start_byte as isize);
            let whitespace_len = scanned_text_fragment_info.range.length() - whitespace_start;
            let mut whitespace_range = Range::new(whitespace_start, whitespace_len);
            whitespace_range.shift_by(scanned_text_fragment_info.range.begin());

            let text_bounds = scanned_text_fragment_info
                .run
                .metrics_for_range(&whitespace_range)
                .bounding_box;
            border_box.size.inline -= text_bounds.size.width;
            scanned_text_fragment_info.content_size.inline -= text_bounds.size.width;

            scanned_text_fragment_info.range.extend_by(-whitespace_len);
            WhitespaceStrippingResult::RetainFragment
        }
    }

    /// Returns the inline-size of this fragment's margin box.
    pub fn margin_box_inline_size(&self) -> Au {
        self.border_box.size.inline + self.margin.inline_start_end()
    }

    /// Returns true if this node *or any of the nodes within its inline fragment context* have
    /// non-`static` `position`.
    pub fn is_positioned(&self) -> bool {
        if self.style.get_box().position != Position::Static {
            return true;
        }
        false
    }

    /// Returns true if this node is absolutely positioned.
    pub fn is_absolutely_positioned(&self) -> bool {
        self.style.get_box().position == Position::Absolute
    }

    pub fn is_text_or_replaced(&self) -> bool {
        match self.specific {
            SpecificFragmentInfo::Generic => false,
            SpecificFragmentInfo::Canvas(_) |
            SpecificFragmentInfo::Iframe(_) |
            SpecificFragmentInfo::Image(_) |
            SpecificFragmentInfo::Media(_) |
            SpecificFragmentInfo::ScannedText(_) |
            SpecificFragmentInfo::TruncatedFragment(_) |
            SpecificFragmentInfo::Svg(_) |
            SpecificFragmentInfo::UnscannedText(_) => true,
        }
    }

    /// Returns the 4D matrix representing this fragment's transform.
    pub fn transform_matrix(
        &self,
        stacking_relative_border_box: &Rect<Au>,
    ) -> Option<LayoutTransform> {
        let list = &self.style.get_box().transform;
        let transform = LayoutTransform::from_untyped(
            &list
                .to_transform_3d_matrix(Some(stacking_relative_border_box))
                .ok()?
                .0,
        );

        let transform_origin = &self.style.get_box().transform_origin;
        let transform_origin_x = transform_origin
            .horizontal
            .to_used_value(stacking_relative_border_box.size.width)
            .to_f32_px();
        let transform_origin_y = transform_origin
            .vertical
            .to_used_value(stacking_relative_border_box.size.height)
            .to_f32_px();
        let transform_origin_z = transform_origin.depth.px();

        let pre_transform = LayoutTransform::create_translation(
            transform_origin_x,
            transform_origin_y,
            transform_origin_z,
        );
        let post_transform = LayoutTransform::create_translation(
            -transform_origin_x,
            -transform_origin_y,
            -transform_origin_z,
        );

        Some(pre_transform.pre_mul(&transform).pre_mul(&post_transform))
    }
}

impl fmt::Debug for Fragment {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let border_padding_string = if !self.border_padding.is_zero() {
            format!("\nborder_padding={:?}", self.border_padding)
        } else {
            "".to_owned()
        };

        let margin_string = if !self.margin.is_zero() {
            format!("\nmargin={:?}", self.margin)
        } else {
            "".to_owned()
        };

        let damage_string = if self.restyle_damage != RestyleDamage::empty() {
            format!("\ndamage={:?}", self.restyle_damage)
        } else {
            "".to_owned()
        };

        write!(
            f,
            "\n{}({}) [{:?}]\nborder_box={:?}{}{}{}",
            self.specific.get_type(),
            self.debug_id,
            self.specific,
            self.border_box,
            border_padding_string,
            margin_string,
            damage_string
        )
    }
}

bitflags! {
    struct QuantitiesIncludedInIntrinsicInlineSizes: u8 {
        const INTRINSIC_INLINE_SIZE_INCLUDES_MARGINS = 0x01;
        const INTRINSIC_INLINE_SIZE_INCLUDES_PADDING = 0x02;
        const INTRINSIC_INLINE_SIZE_INCLUDES_BORDER = 0x04;
        const INTRINSIC_INLINE_SIZE_INCLUDES_SPECIFIED = 0x08;
    }
}

bitflags! {
    // Various flags we can use when splitting fragments. See
    // `calculate_split_position_using_breaking_strategy()`.
    struct SplitOptions: u8 {
        #[doc = "True if this is the first fragment on the line."]
        const STARTS_LINE = 0x01;
        #[doc = "True if we should attempt to split at character boundaries if this split fails. \
                 This is used to implement `overflow-wrap: break-word`."]
        const RETRY_AT_CHARACTER_BOUNDARIES = 0x02;
    }
}

/// A top-down fragment border box iteration handler.
pub trait FragmentBorderBoxIterator {
    /// The operation to perform.
    fn process(&mut self, fragment: &Fragment, level: i32, overflow: &Rect<Au>);

    /// Returns true if this fragment must be processed in-order. If this returns false,
    /// we skip the operation for this fragment, but continue processing siblings.
    fn should_process(&mut self, fragment: &Fragment) -> bool;
}

/// The coordinate system used in `stacking_relative_border_box()`. See the documentation of that
/// method for details.
#[derive(Clone, Debug, PartialEq)]
pub enum CoordinateSystem {
    /// The border box returned is relative to the fragment's parent stacking context.
    Parent,
    /// The border box returned is relative to the fragment's own stacking context, if applicable.
    Own,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum WhitespaceStrippingResult {
    RetainFragment,
    FragmentContainedOnlyBidiControlCharacters,
    FragmentContainedOnlyWhitespace,
}

impl WhitespaceStrippingResult {
    fn from_unscanned_text_fragment_info(
        info: &UnscannedTextFragmentInfo,
    ) -> WhitespaceStrippingResult {
        if info.text.is_empty() {
            WhitespaceStrippingResult::FragmentContainedOnlyWhitespace
        } else if info.text.chars().all(gfx::text::util::is_bidi_control) {
            WhitespaceStrippingResult::FragmentContainedOnlyBidiControlCharacters
        } else {
            WhitespaceStrippingResult::RetainFragment
        }
    }
}

/// The overflow area. We need two different notions of overflow: paint overflow and scrollable
/// overflow.
#[derive(Clone, Copy, Debug)]
pub struct Overflow {
    pub scroll: Rect<Au>,
    pub paint: Rect<Au>,
}

impl Overflow {
    pub fn new() -> Overflow {
        Overflow {
            scroll: Rect::zero(),
            paint: Rect::zero(),
        }
    }

    pub fn from_rect(border_box: &Rect<Au>) -> Overflow {
        Overflow {
            scroll: *border_box,
            paint: *border_box,
        }
    }

    pub fn union(&mut self, other: &Overflow) {
        self.scroll = self.scroll.union(&other.scroll);
        self.paint = self.paint.union(&other.paint);
    }

    pub fn translate(&mut self, by: &Vector2D<Au>) {
        self.scroll = self.scroll.translate(by);
        self.paint = self.paint.translate(by);
    }
}

bitflags! {
    pub struct FragmentFlags: u8 {
        // TODO(stshine): find a better name since these flags can also be used for grid item.
        /// Whether this fragment represents a child in a row flex container.
        const IS_INLINE_FLEX_ITEM = 0b0000_0001;
        /// Whether this fragment represents a child in a column flex container.
        const IS_BLOCK_FLEX_ITEM = 0b0000_0010;
        /// Whether this fragment represents the generated text from a text-overflow clip.
        const IS_ELLIPSIS = 0b0000_0100;
    }
}

/// Specified distances from the margin edge of a block to its content in the inline direction.
/// These are returned by `guess_inline_content_edge_offsets()` and are used in the float placement
/// speculation logic.
#[derive(Clone, Copy, Debug)]
pub struct SpeculatedInlineContentEdgeOffsets {
    pub start: Au,
    pub end: Au,
}

#[cfg(not(debug_assertions))]
#[derive(Clone)]
struct DebugId;

#[cfg(debug_assertions)]
#[derive(Clone)]
struct DebugId(u16);

#[cfg(not(debug_assertions))]
impl DebugId {
    pub fn new() -> DebugId {
        DebugId
    }
}

#[cfg(debug_assertions)]
impl DebugId {
    pub fn new() -> DebugId {
        DebugId(layout_debug::generate_unique_debug_id())
    }
}

#[cfg(not(debug_assertions))]
impl fmt::Display for DebugId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:p}", &self)
    }
}

#[cfg(debug_assertions)]
impl fmt::Display for DebugId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(not(debug_assertions))]
impl Serialize for DebugId {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&format!("{:p}", &self))
    }
}

#[cfg(debug_assertions)]
impl Serialize for DebugId {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u16(self.0)
    }
}
