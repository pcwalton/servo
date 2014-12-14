/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![allow(non_camel_case_types)]

use url::{Url, UrlParser};

pub use servo_util::geometry::Au;

pub type CSSFloat = f64;

pub mod specified {
    use std::ascii::AsciiExt;
    use std::f64::consts::PI;
    use url::Url;
    use cssparser::ast;
    use cssparser::ast::*;
    use parsing_utils::{mod, BufferedIter, ParserIter};
    use super::{Au, CSSFloat};
    pub use cssparser::Color as CSSColor;

    #[deriving(Clone, Show)]
    pub enum Length {
        Au_(Au),  // application units
        Em(CSSFloat),
        Ex(CSSFloat),
        Rem(CSSFloat),

        /// HTML5 "character width", as defined in HTML5 § 14.5.4.
        ///
        /// This cannot be specified by the user directly and is only generated by
        /// `Stylist::synthesize_rules_for_legacy_attributes()`.
        ServoCharacterWidth(i32),

        // XXX uncomment when supported:
//        Ch(CSSFloat),
//        Vw(CSSFloat),
//        Vh(CSSFloat),
//        Vmin(CSSFloat),
//        Vmax(CSSFloat),
    }
    const AU_PER_PX: CSSFloat = 60.;
    const AU_PER_IN: CSSFloat = AU_PER_PX * 96.;
    const AU_PER_CM: CSSFloat = AU_PER_IN / 2.54;
    const AU_PER_MM: CSSFloat = AU_PER_IN / 25.4;
    const AU_PER_PT: CSSFloat = AU_PER_IN / 72.;
    const AU_PER_PC: CSSFloat = AU_PER_PT * 12.;
    impl Length {
        #[inline]
        fn parse_internal(input: &ComponentValue, negative_ok: bool) -> Result<Length, ()> {
            match input {
                &Dimension(ref value, ref unit) if negative_ok || value.value >= 0.
                => Length::parse_dimension(value.value, unit.as_slice()),
                &Number(ref value) if value.value == 0. =>  Ok(Au_(Au(0))),
                _ => Err(())
            }
        }
        #[allow(dead_code)]
        pub fn parse(input: &ComponentValue) -> Result<Length, ()> {
            Length::parse_internal(input, /* negative_ok = */ true)
        }
        pub fn parse_non_negative(input: &ComponentValue) -> Result<Length, ()> {
            Length::parse_internal(input, /* negative_ok = */ false)
        }
        pub fn parse_dimension(value: CSSFloat, unit: &str) -> Result<Length, ()> {
            match unit.to_ascii_lower().as_slice() {
                "px" => Ok(Length::from_px(value)),
                "in" => Ok(Au_(Au((value * AU_PER_IN) as i32))),
                "cm" => Ok(Au_(Au((value * AU_PER_CM) as i32))),
                "mm" => Ok(Au_(Au((value * AU_PER_MM) as i32))),
                "pt" => Ok(Au_(Au((value * AU_PER_PT) as i32))),
                "pc" => Ok(Au_(Au((value * AU_PER_PC) as i32))),
                "em" => Ok(Em(value)),
                "ex" => Ok(Ex(value)),
                "rem" => Ok(Rem(value)),
                _ => Err(())
            }
        }
        #[inline]
        pub fn from_px(px_value: CSSFloat) -> Length {
            Au_(Au((px_value * AU_PER_PX) as i32))
        }
    }

    #[deriving(Clone, Show)]
    pub enum LengthOrPercentage {
        LP_Length(Length),
        LP_Percentage(CSSFloat),  // [0 .. 100%] maps to [0.0 .. 1.0]
    }

    impl LengthOrPercentage {
        fn parse_internal(input: &ComponentValue, negative_ok: bool)
                              -> Result<LengthOrPercentage, ()> {
            match input {
                &Dimension(ref value, ref unit) if negative_ok || value.value >= 0.
                => Length::parse_dimension(value.value, unit.as_slice()).map(LP_Length),
                &ast::Percentage(ref value) if negative_ok || value.value >= 0.
                => Ok(LP_Percentage(value.value / 100.)),
                &Number(ref value) if value.value == 0. =>  Ok(LP_Length(Au_(Au(0)))),
                _ => Err(())
            }
        }
        #[allow(dead_code)]
        #[inline]
        pub fn parse(input: &ComponentValue) -> Result<LengthOrPercentage, ()> {
            LengthOrPercentage::parse_internal(input, /* negative_ok = */ true)
        }
        #[inline]
        pub fn parse_non_negative(input: &ComponentValue) -> Result<LengthOrPercentage, ()> {
            LengthOrPercentage::parse_internal(input, /* negative_ok = */ false)
        }
    }

    #[deriving(Clone)]
    pub enum LengthOrPercentageOrAuto {
        LPA_Length(Length),
        LPA_Percentage(CSSFloat),  // [0 .. 100%] maps to [0.0 .. 1.0]
        LPA_Auto,
    }
    impl LengthOrPercentageOrAuto {
        fn parse_internal(input: &ComponentValue, negative_ok: bool)
                     -> Result<LengthOrPercentageOrAuto, ()> {
            match input {
                &Dimension(ref value, ref unit) if negative_ok || value.value >= 0.
                => Length::parse_dimension(value.value, unit.as_slice()).map(LPA_Length),
                &ast::Percentage(ref value) if negative_ok || value.value >= 0.
                => Ok(LPA_Percentage(value.value / 100.)),
                &Number(ref value) if value.value == 0. => Ok(LPA_Length(Au_(Au(0)))),
                &Ident(ref value) if value.as_slice().eq_ignore_ascii_case("auto") => Ok(LPA_Auto),
                _ => Err(())
            }
        }
        #[inline]
        pub fn parse(input: &ComponentValue) -> Result<LengthOrPercentageOrAuto, ()> {
            LengthOrPercentageOrAuto::parse_internal(input, /* negative_ok = */ true)
        }
        #[inline]
        pub fn parse_non_negative(input: &ComponentValue) -> Result<LengthOrPercentageOrAuto, ()> {
            LengthOrPercentageOrAuto::parse_internal(input, /* negative_ok = */ false)
        }
    }

    #[deriving(Clone)]
    pub enum LengthOrPercentageOrNone {
        Length(Length),
        Percentage(CSSFloat),  // [0 .. 100%] maps to [0.0 .. 1.0]
        None,
    }
    impl LengthOrPercentageOrNone {
        fn parse_internal(input: &ComponentValue, negative_ok: bool)
                     -> Result<LengthOrPercentageOrNone, ()> {
            match input {
                &Dimension(ref value, ref unit) if negative_ok || value.value >= 0.
                => Length::parse_dimension(value.value, unit.as_slice()).map(LengthOrPercentageOrNone::Length),
                &ast::Percentage(ref value) if negative_ok || value.value >= 0.
                => Ok(LengthOrPercentageOrNone::Percentage(value.value / 100.)),
                &Number(ref value) if value.value == 0. => Ok(LengthOrPercentageOrNone::Length(Au_(Au(0)))),
                &Ident(ref value) if value.as_slice().eq_ignore_ascii_case("none") => Ok(LengthOrPercentageOrNone::None),
                _ => Err(())
            }
        }
        #[allow(dead_code)]
        #[inline]
        pub fn parse(input: &ComponentValue) -> Result<LengthOrPercentageOrNone, ()> {
            LengthOrPercentageOrNone::parse_internal(input, /* negative_ok = */ true)
        }
        #[inline]
        pub fn parse_non_negative(input: &ComponentValue) -> Result<LengthOrPercentageOrNone, ()> {
            LengthOrPercentageOrNone::parse_internal(input, /* negative_ok = */ false)
        }
    }

    // http://dev.w3.org/csswg/css2/colors.html#propdef-background-position
    #[deriving(Clone)]
    pub enum PositionComponent {
        Pos_Length(Length),
        Pos_Percentage(CSSFloat),  // [0 .. 100%] maps to [0.0 .. 1.0]
        Pos_Center,
        Pos_Left,
        Pos_Right,
        Pos_Top,
        Pos_Bottom,
    }
    impl PositionComponent {
        pub fn parse(input: &ComponentValue) -> Result<PositionComponent, ()> {
            match input {
                &Dimension(ref value, ref unit) =>
                    Length::parse_dimension(value.value, unit.as_slice()).map(Pos_Length),
                &ast::Percentage(ref value) => Ok(Pos_Percentage(value.value / 100.)),
                &Number(ref value) if value.value == 0. => Ok(Pos_Length(Au_(Au(0)))),
                &Ident(ref value) => {
                    if value.as_slice().eq_ignore_ascii_case("center") { Ok(Pos_Center) }
                    else if value.as_slice().eq_ignore_ascii_case("left") { Ok(Pos_Left) }
                    else if value.as_slice().eq_ignore_ascii_case("right") { Ok(Pos_Right) }
                    else if value.as_slice().eq_ignore_ascii_case("top") { Ok(Pos_Top) }
                    else if value.as_slice().eq_ignore_ascii_case("bottom") { Ok(Pos_Bottom) }
                    else { Err(()) }
                }
                _ => Err(())
            }
        }
        #[inline]
        pub fn to_length_or_percentage(self) -> LengthOrPercentage {
            match self {
                Pos_Length(x) => LP_Length(x),
                Pos_Percentage(x) => LP_Percentage(x),
                Pos_Center => LP_Percentage(0.5),
                Pos_Left | Pos_Top => LP_Percentage(0.0),
                Pos_Right | Pos_Bottom => LP_Percentage(1.0),
            }
        }
    }

    #[deriving(Clone, PartialEq, PartialOrd)]
    pub struct Angle(pub CSSFloat);

    impl Angle {
        pub fn radians(self) -> f64 {
            let Angle(radians) = self;
            radians
        }
    }

    static DEG_TO_RAD: CSSFloat = PI / 180.0;
    static GRAD_TO_RAD: CSSFloat = PI / 200.0;

    impl Angle {
        /// Parses an angle according to CSS-VALUES § 6.1.
        fn parse_dimension(value: CSSFloat, unit: &str) -> Result<Angle,()> {
            if unit.eq_ignore_ascii_case("deg") {
                Ok(Angle(value * DEG_TO_RAD))
            } else if unit.eq_ignore_ascii_case("grad") {
                Ok(Angle(value * GRAD_TO_RAD))
            } else if unit.eq_ignore_ascii_case("rad") {
                Ok(Angle(value))
            } else if unit.eq_ignore_ascii_case("turn") {
                Ok(Angle(value * 2.0 * PI))
            } else {
                Err(())
            }
        }
    }

    /// Specified values for an image according to CSS-IMAGES.
    #[deriving(Clone)]
    pub enum Image {
        UrlImage(Url),
        LinearGradientImage(LinearGradient),
    }

    impl Image {
        pub fn from_component_value(component_value: &ComponentValue, base_url: &Url)
                                    -> Result<Image,()> {
            match component_value {
                &ast::URL(ref url) => {
                    let image_url = super::parse_url(url.as_slice(), base_url);
                    Ok(UrlImage(image_url))
                },
                &ast::Function(ref name, ref args) => {
                    if name.as_slice().eq_ignore_ascii_case("linear-gradient") {
                        Ok(LinearGradientImage(try!(
                                    super::specified::LinearGradient::parse_function(
                                    args.as_slice()))))
                    } else {
                        Err(())
                    }
                }
                _ => Err(()),
            }
        }

        pub fn to_computed_value(self, context: &super::computed::Context)
                                 -> super::computed::Image {
            match self {
                UrlImage(url) => super::computed::UrlImage(url),
                LinearGradientImage(linear_gradient) => {
                    super::computed::LinearGradientImage(
                        super::computed::LinearGradient::compute(linear_gradient, context))
                }
            }
        }
    }

    /// Specified values for a CSS linear gradient.
    #[deriving(Clone)]
    pub struct LinearGradient {
        /// The angle or corner of the gradient.
        pub angle_or_corner: AngleOrCorner,

        /// The color stops.
        pub stops: Vec<ColorStop>,
    }

    /// Specified values for an angle or a corner in a linear gradient.
    #[deriving(Clone, PartialEq)]
    pub enum AngleOrCorner {
        Angle(Angle),
        Corner(HorizontalDirection, VerticalDirection),
    }

    /// Specified values for one color stop in a linear gradient.
    #[deriving(Clone)]
    pub struct ColorStop {
        /// The color of this stop.
        pub color: CSSColor,

        /// The position of this stop. If not specified, this stop is placed halfway between the
        /// point that precedes it and the point that follows it.
        pub position: Option<LengthOrPercentage>,
    }

    #[deriving(Clone, PartialEq)]
    pub enum HorizontalDirection {
        Left,
        Right,
    }

    #[deriving(Clone, PartialEq)]
    pub enum VerticalDirection {
        Top,
        Bottom,
    }

    fn parse_color_stop(source: ParserIter) -> Result<ColorStop,()> {
        let color = match source.next() {
            Some(color) => try!(CSSColor::parse(color)),
            None => return Err(()),
        };

        let position = match source.next() {
            None => None,
            Some(value) => {
                match *value {
                    Comma => {
                        source.push_back(value);
                        None
                    }
                    ref position => Some(try!(LengthOrPercentage::parse(position))),
                }
            }
        };

        Ok(ColorStop {
            color: color,
            position: position,
        })
    }

    impl LinearGradient {
        /// Parses a linear gradient from the given arguments.
        pub fn parse_function(args: &[ComponentValue]) -> Result<LinearGradient,()> {
            let mut source = BufferedIter::new(args.skip_whitespace());

            // Parse the angle.
            let (angle_or_corner, need_to_parse_comma) = match source.next() {
                None => return Err(()),
                Some(token) => {
                    match *token {
                        Dimension(ref value, ref unit) => {
                            match Angle::parse_dimension(value.value, unit.as_slice()) {
                                Ok(angle) => {
                                    (AngleOrCorner::Angle(angle), true)
                                }
                                Err(()) => {
                                    source.push_back(token);
                                    (AngleOrCorner::Angle(Angle(PI)), false)
                                }
                            }
                        }
                        Ident(ref ident) if ident.as_slice().eq_ignore_ascii_case("to") => {
                            let (mut horizontal, mut vertical) = (None, None);
                            loop {
                                match source.next() {
                                    None => break,
                                    Some(token) => {
                                        match *token {
                                            Ident(ref ident) => {
                                                let ident = ident.as_slice();
                                                if ident.eq_ignore_ascii_case("top") &&
                                                        vertical.is_none() {
                                                    vertical = Some(Top)
                                                } else if ident.eq_ignore_ascii_case("bottom") &&
                                                        vertical.is_none() {
                                                    vertical = Some(Bottom)
                                                } else if ident.eq_ignore_ascii_case("left") &&
                                                        horizontal.is_none() {
                                                    horizontal = Some(Left)
                                                } else if ident.eq_ignore_ascii_case("right") &&
                                                        horizontal.is_none() {
                                                    horizontal = Some(Right)
                                                } else {
                                                    return Err(())
                                                }
                                            }
                                            Comma => {
                                                source.push_back(token);
                                                break
                                            }
                                            _ => return Err(()),
                                        }
                                    }
                                }
                            }

                            (match (horizontal, vertical) {
                                (None, Some(Top)) => AngleOrCorner::Angle(Angle(0.0)),
                                (Some(Right), None) => AngleOrCorner::Angle(Angle(PI * 0.5)),
                                (None, Some(Bottom)) => AngleOrCorner::Angle(Angle(PI)),
                                (Some(Left), None) => AngleOrCorner::Angle(Angle(PI * 1.5)),
                                (Some(horizontal), Some(vertical)) => {
                                    AngleOrCorner::Corner(horizontal, vertical)
                                }
                                (None, None) => return Err(()),
                            }, true)
                        }
                        _ => {
                            source.push_back(token);
                            (AngleOrCorner::Angle(Angle(PI)), false)
                        }
                    }
                }
            };

            // Parse the color stops.
            let stops = if need_to_parse_comma {
                match source.next() {
                    Some(&Comma) => {
                        try!(parsing_utils::parse_comma_separated(&mut source, parse_color_stop))
                    }
                    None => Vec::new(),
                    Some(_) => return Err(()),
                }
            } else {
                try!(parsing_utils::parse_comma_separated(&mut source, parse_color_stop))
            };

            if stops.len() < 2 {
                return Err(())
            }

            Ok(LinearGradient {
                angle_or_corner: angle_or_corner,
                stops: stops,
            })
        }
    }
}

pub mod computed {
    pub use super::specified::{Angle, AngleOrCorner, HorizontalDirection};
    pub use super::specified::{VerticalDirection};
    pub use cssparser::Color as CSSColor;
    pub use super::super::longhands::computed_as_specified as compute_CSSColor;
    use super::*;
    use super::super::longhands;
    use url::Url;

    pub struct Context {
        pub inherited_font_weight: longhands::font_weight::computed_value::T,
        pub inherited_font_size: longhands::font_size::computed_value::T,
        pub inherited_text_decorations_in_effect: longhands::_servo_text_decorations_in_effect::T,
        pub inherited_height: longhands::height::T,
        pub color: longhands::color::computed_value::T,
        pub text_decoration: longhands::text_decoration::computed_value::T,
        pub font_size: longhands::font_size::computed_value::T,
        pub root_font_size: longhands::font_size::computed_value::T,
        pub display: longhands::display::computed_value::T,
        pub positioned: bool,
        pub floated: bool,
        pub border_top_present: bool,
        pub border_right_present: bool,
        pub border_bottom_present: bool,
        pub border_left_present: bool,
        pub is_root_element: bool,
        // TODO, as needed: viewport size, etc.
    }

    #[allow(non_snake_case)]
    #[inline]
    pub fn compute_Au(value: specified::Length, context: &Context) -> Au {
        compute_Au_with_font_size(value, context.font_size, context.root_font_size)
    }

    /// A special version of `compute_Au` used for `font-size`.
    #[allow(non_snake_case)]
    #[inline]
    pub fn compute_Au_with_font_size(value: specified::Length, reference_font_size: Au, root_font_size: Au) -> Au {
        match value {
            specified::Au_(value) => value,
            specified::Em(value) => reference_font_size.scale_by(value),
            specified::Ex(value) => {
                let x_height = 0.5;  // TODO: find that from the font
                reference_font_size.scale_by(value * x_height)
            },
            specified::Rem(value) => root_font_size.scale_by(value),
            specified::ServoCharacterWidth(value) => {
                // This applies the *converting a character width to pixels* algorithm as specified
                // in HTML5 § 14.5.4.
                //
                // TODO(pcwalton): Find these from the font.
                let average_advance = reference_font_size.scale_by(0.5);
                let max_advance = reference_font_size;
                average_advance.scale_by(value as CSSFloat - 1.0) + max_advance
            }
        }
    }

    #[deriving(PartialEq, Clone, Show)]
    pub enum LengthOrPercentage {
        LP_Length(Au),
        LP_Percentage(CSSFloat),
    }

    #[allow(non_snake_case)]
    pub fn compute_LengthOrPercentage(value: specified::LengthOrPercentage, context: &Context)
                                   -> LengthOrPercentage {
        match value {
            specified::LP_Length(value) => LP_Length(compute_Au(value, context)),
            specified::LP_Percentage(value) => LP_Percentage(value),
        }
    }

    #[deriving(PartialEq, Clone, Show)]
    pub enum LengthOrPercentageOrAuto {
        LPA_Length(Au),
        LPA_Percentage(CSSFloat),
        LPA_Auto,
    }
    #[allow(non_snake_case)]
    pub fn compute_LengthOrPercentageOrAuto(value: specified::LengthOrPercentageOrAuto,
                                            context: &Context) -> LengthOrPercentageOrAuto {
        match value {
            specified::LPA_Length(value) => LPA_Length(compute_Au(value, context)),
            specified::LPA_Percentage(value) => LPA_Percentage(value),
            specified::LPA_Auto => LPA_Auto,
        }
    }

    #[deriving(PartialEq, Clone, Show)]
    pub enum LengthOrPercentageOrNone {
        LPN_Length(Au),
        LPN_Percentage(CSSFloat),
        LPN_None,
    }
    #[allow(non_snake_case)]
    pub fn compute_LengthOrPercentageOrNone(value: specified::LengthOrPercentageOrNone,
                                            context: &Context) -> LengthOrPercentageOrNone {
        match value {
            specified::LengthOrPercentageOrNone::Length(value) => LPN_Length(compute_Au(value, context)),
            specified::LengthOrPercentageOrNone::Percentage(value) => LPN_Percentage(value),
            specified::LengthOrPercentageOrNone::None => LPN_None,
        }
    }

    /// Computed values for an image according to CSS-IMAGES.
    #[deriving(Clone, PartialEq)]
    pub enum Image {
        UrlImage(Url),
        LinearGradientImage(LinearGradient),
    }

    /// Computed values for a CSS linear gradient.
    #[deriving(Clone, PartialEq)]
    pub struct LinearGradient {
        /// The angle or corner of the gradient.
        pub angle_or_corner: AngleOrCorner,

        /// The color stops.
        pub stops: Vec<ColorStop>,
    }

    /// Computed values for one color stop in a linear gradient.
    #[deriving(Clone, PartialEq)]
    pub struct ColorStop {
        /// The color of this stop.
        pub color: CSSColor,

        /// The position of this stop. If not specified, this stop is placed halfway between the
        /// point that precedes it and the point that follows it per CSS-IMAGES § 3.4.
        pub position: Option<LengthOrPercentage>,
    }

    impl LinearGradient {
        pub fn compute(value: specified::LinearGradient, context: &Context) -> LinearGradient {
            let specified::LinearGradient {
                angle_or_corner,
                stops
            } = value;
            LinearGradient {
                angle_or_corner: angle_or_corner,
                stops: stops.into_iter().map(|stop| {
                    ColorStop {
                        color: stop.color,
                        position: match stop.position {
                            None => None,
                            Some(value) => Some(compute_LengthOrPercentage(value, context)),
                        },
                    }
                }).collect()
            }
        }
    }
}

pub fn parse_url(input: &str, base_url: &Url) -> Url {
    UrlParser::new().base_url(base_url).parse(input)
        .unwrap_or_else(|_| Url::parse("about:invalid").unwrap())
}
