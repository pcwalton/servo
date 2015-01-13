/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::CanvasRenderingContext2DBinding;
use dom::bindings::codegen::Bindings::CanvasRenderingContext2DBinding::CanvasRenderingContext2DMethods;
use dom::bindings::codegen::Bindings::CanvasRenderingContext2DBinding::CanvasWindingRule;
use dom::bindings::codegen::UnionTypes::StringOrCanvasGradientOrCanvasPattern;
use dom::bindings::global::{GlobalRef, GlobalField};
use dom::bindings::js::{JS, JSRef, Temporary};
use dom::bindings::utils::{Reflector, reflect_dom_object};
use dom::htmlcanvaselement::HTMLCanvasElement;

use canvas::canvas_paint_task::{CanvasMsg, CanvasPaintTask, FillOrStrokeStyle};
use cssparser::Color as CSSColor;
use cssparser::{Parser, RGBA, ToCss};
use geom::matrix2d::Matrix2D;
use geom::point::Point2D;
use geom::rect::Rect;
use geom::size::Size2D;
use std::cell::Cell;
use std::sync::mpsc::Sender;

#[dom_struct]
pub struct CanvasRenderingContext2D {
    reflector_: Reflector,
    global: GlobalField,
    renderer: Sender<CanvasMsg>,
    canvas: JS<HTMLCanvasElement>,
    stroke_color: Cell<RGBA>,
    fill_color: Cell<RGBA>,
    transform: Cell<Matrix2D<f32>>,
}

impl CanvasRenderingContext2D {
    fn new_inherited(global: GlobalRef, canvas: JSRef<HTMLCanvasElement>, size: Size2D<i32>)
                     -> CanvasRenderingContext2D {
        let black = RGBA {
            red: 0.0,
            green: 0.0,
            blue: 0.0,
            alpha: 1.0,
        };
        CanvasRenderingContext2D {
            reflector_: Reflector::new(),
            global: GlobalField::from_rooted(&global),
            renderer: CanvasPaintTask::start(size),
            canvas: JS::from_rooted(canvas),
            stroke_color: Cell::new(black),
            fill_color: Cell::new(black),
            transform: Cell::new(Matrix2D::identity()),
        }
    }

    pub fn new(global: GlobalRef, canvas: JSRef<HTMLCanvasElement>, size: Size2D<i32>)
               -> Temporary<CanvasRenderingContext2D> {
        reflect_dom_object(box CanvasRenderingContext2D::new_inherited(global, canvas, size),
                           global, CanvasRenderingContext2DBinding::Wrap)
    }

    pub fn recreate(&self, size: Size2D<i32>) {
        self.renderer.send(CanvasMsg::Recreate(size)).unwrap();
    }

    fn update_transform(&self) {
        self.renderer.send(CanvasMsg::SetTransform(self.transform.get())).unwrap()
    }
}

pub trait LayoutCanvasRenderingContext2DHelpers {
    unsafe fn get_renderer(&self) -> Sender<CanvasMsg>;
}

impl LayoutCanvasRenderingContext2DHelpers for JS<CanvasRenderingContext2D> {
    unsafe fn get_renderer(&self) -> Sender<CanvasMsg> {
        (*self.unsafe_get()).renderer.clone()
    }
}

impl<'a> CanvasRenderingContext2DMethods for JSRef<'a, CanvasRenderingContext2D> {
    fn Canvas(self) -> Temporary<HTMLCanvasElement> {
        Temporary::new(self.canvas)
    }

    fn Scale(self, x: f64, y: f64) {
        self.transform.set(self.transform.get().scale(x as f32, y as f32));
        self.update_transform()
    }

    fn Translate(self, x: f64, y: f64) {
        self.transform.set(self.transform.get().translate(x as f32, y as f32));
        self.update_transform()
    }

    fn Transform(self, a: f64, b: f64, c: f64, d: f64, e: f64, f: f64) {
        self.transform.set(self.transform.get().mul(&Matrix2D::new(a as f32,
                                                                   b as f32,
                                                                   c as f32,
                                                                   d as f32,
                                                                   e as f32,
                                                                   f as f32)));
        self.update_transform()
    }

    fn SetTransform(self, a: f64, b: f64, c: f64, d: f64, e: f64, f: f64) {
        self.transform.set(Matrix2D::new(a as f32,
                                         b as f32,
                                         c as f32,
                                         d as f32,
                                         e as f32,
                                         f as f32));
        self.update_transform()
    }

    fn FillRect(self, x: f64, y: f64, width: f64, height: f64) {
        let rect = Rect(Point2D(x as f32, y as f32), Size2D(width as f32, height as f32));
        self.renderer.send(CanvasMsg::FillRect(rect));
    }

    fn ClearRect(self, x: f64, y: f64, width: f64, height: f64) {
        let rect = Rect(Point2D(x as f32, y as f32), Size2D(width as f32, height as f32));
        self.renderer.send(CanvasMsg::ClearRect(rect));
    }

    fn StrokeRect(self, x: f64, y: f64, width: f64, height: f64) {
        let rect = Rect(Point2D(x as f32, y as f32), Size2D(width as f32, height as f32));
        self.renderer.send(CanvasMsg::StrokeRect(rect));
    }

    fn BeginPath(self) {
        self.renderer.send(CanvasMsg::BeginPath);
    }

    fn ClosePath(self) {
        self.renderer.send(CanvasMsg::ClosePath);
    }

    fn Fill(self, _: CanvasWindingRule) {
        self.renderer.send(CanvasMsg::Fill);
    }

    fn MoveTo(self, x: f64, y: f64) {
        self.renderer.send(CanvasMsg::MoveTo(Point2D(x as f32, y as f32)));
    }

    fn BezierCurveTo(self, cp1x: f64, cp1y: f64, cp2x: f64, cp2y: f64, x: f64, y: f64) {
        self.renderer.send(CanvasMsg::BezierCurveTo(Point2D(cp1x as f32, cp1y as f32),
                                                    Point2D(cp2x as f32, cp2y as f32),
                                                    Point2D(x as f32, y as f32)));
    }

    fn StrokeStyle(self) -> StringOrCanvasGradientOrCanvasPattern {
        // FIXME(pcwalton, #4761): This is not spec-compliant. See:
        //
        // https://html.spec.whatwg.org/multipage/scripting.html#serialisation-of-a-colour
        let mut result = String::new();
        self.stroke_color.get().to_css(&mut result).unwrap();
        StringOrCanvasGradientOrCanvasPattern::eString(result)
    }

    fn SetStrokeStyle(self, value: StringOrCanvasGradientOrCanvasPattern) {
        match value {
            StringOrCanvasGradientOrCanvasPattern::eString(string) => {
                match parse_color(string.as_slice()) {
                    Ok(rgba) => {
                        self.stroke_color.set(rgba);
                        self.renderer
                            .send(CanvasMsg::SetStrokeStyle(FillOrStrokeStyle::Color(rgba)))
                            .unwrap();
                    }
                    _ => {}
                }
            }
            _ => {
                // TODO(pcwalton)
            }
        }
    }

    fn FillStyle(self) -> StringOrCanvasGradientOrCanvasPattern {
        // FIXME(pcwalton, #4761): This is not spec-compliant. See:
        //
        // https://html.spec.whatwg.org/multipage/scripting.html#serialisation-of-a-colour
        let mut result = String::new();
        self.stroke_color.get().to_css(&mut result).unwrap();
        StringOrCanvasGradientOrCanvasPattern::eString(result)
    }

    fn SetFillStyle(self, value: StringOrCanvasGradientOrCanvasPattern) {
        match value {
            StringOrCanvasGradientOrCanvasPattern::eString(string) => {
                match parse_color(string.as_slice()) {
                    Ok(rgba) => {
                        self.fill_color.set(rgba);
                        self.renderer
                            .send(CanvasMsg::SetFillStyle(FillOrStrokeStyle::Color(rgba)))
                            .unwrap()
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
}

#[unsafe_destructor]
impl Drop for CanvasRenderingContext2D {
    fn drop(&mut self) {
        self.renderer.send(CanvasMsg::Close);
    }
}

pub fn parse_color(string: &str) -> Result<RGBA,()> {
    match CSSColor::parse(&mut Parser::new(string.as_slice())) {
        Ok(CSSColor::RGBA(rgba)) => Ok(rgba),
        _ => Err(()),
    }
}

