/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The task that handles all rendering/painting.

use azure::azure_hl::{B8G8R8A8, DrawTarget};
use azure::{AzFloat, AzGLContext};
use display_list::DisplayList;
use font_context::FontContext;
use opts::Opts;
use render_context::RenderContext;
use servo_msg::compositor::LayerBufferSet;
use servo_msg::compositor::{RenderListener, IdleRenderState, RenderingRenderState, LayerBuffer};

use core::cast;
use core::cell::Cell;
use core::comm::{Chan, Port, SharedChan};
use geom::matrix2d::Matrix2D;
use geom::point::Point2D;
use geom::rect::Rect;
use geom::size::Size2D;
use layers::layers::ARGB32Format;
use layers::texturegl::{Texture, TextureImageData};
use servo_util::time::{ProfilerChan, RenderingDrawingCategory, profile};
use servo_util::time;
use sharegl::context::GraphicsContextMethods;
use sharegl::platform::GraphicsContext;

pub struct RenderLayer {
    display_list: DisplayList<()>,
    size: Size2D<uint>
}

pub enum Msg<C> {
    AttachCompositorMsg(C),
    RenderMsg(RenderLayer),
    ReRenderMsg(f32),
    ExitMsg(Chan<()>),
}

pub struct RenderChan<C> {
    chan: SharedChan<Msg<C>>,
}

impl<C: RenderListener + Owned> Clone for RenderChan<C> {
    pub fn clone(&self) -> RenderChan<C> {
        RenderChan {
            chan: self.chan.clone(),
        }
    }
}

impl<C: RenderListener + Owned> RenderChan<C> {
    pub fn new(chan: Chan<Msg<C>>) -> RenderChan<C> {
        RenderChan {
            chan: SharedChan::new(chan),
        }
    }
    pub fn send(&self, msg: Msg<C>) {
        self.chan.send(msg);
    }
}

pub fn create_render_task<C: RenderListener + Owned>(port: Port<Msg<C>>,
                                                     compositor: C,
                                                     opts: Opts,
                                                     profiler_chan: ProfilerChan) {
    let compositor_cell = Cell(compositor);
    let opts_cell = Cell(opts);
    let port = Cell(port);

    do spawn {
        let compositor = compositor_cell.take();
        let share_gl_context = unsafe {
            GraphicsContextMethods::wrap(cast::transmute(compositor.get_gl_context()))
        };
        let opts = opts_cell.with_ref(|o| copy *o);
        let profiler_chan = profiler_chan.clone();
        let profiler_chan_copy = profiler_chan.clone();

        // FIXME: rust/#5967
        let mut renderer = Renderer {
            port: port.take(),
            compositor: compositor,
            font_ctx: @mut FontContext::new(opts.render_backend,
                                            false,
                                            profiler_chan),
            opts: opts_cell.take(),
            profiler_chan: profiler_chan_copy,
            share_gl_context: share_gl_context,
            render_layer: None,
        };

        renderer.start();
    }
}

struct Renderer<C> {
    /// A port that receives messages from the compositor.
    port: Port<Msg<C>>,

    /// The interface to the compositor.
    compositor: C,

    /// The font context.
    font_ctx: @mut FontContext,

    /// The command line options passed to Servo.
    opts: Opts,

    /// A channel to the profiler.
    profiler_chan: ProfilerChan,

    /// The 3D graphics context to render with.
    share_gl_context: GraphicsContext,

    /// The layer to be rendered.
    render_layer: Option<RenderLayer>,
}

impl<C: RenderListener + Owned> Renderer<C> {
    fn start(&mut self) {
        debug!("renderer: beginning rendering loop");

        loop {
            match self.port.recv() {
                AttachCompositorMsg(compositor) => self.compositor = compositor,
                RenderMsg(render_layer) => {
                    self.render_layer = Some(render_layer);
                    self.render(1.0);
                }
                ReRenderMsg(scale) => {
                    self.render(scale);
                }
                ExitMsg(response_ch) => {
                    response_ch.send(());
                    break;
                }
            }
        }
    }

    fn render(&mut self, scale: f32) {
        debug!("renderer: rendering");
        
        let render_layer;
        match (self.render_layer) {
            None => return,
            Some(ref r_layer) => {
                render_layer = r_layer;
            }
        }

        self.compositor.set_render_state(RenderingRenderState);
        do profile(time::RenderingCategory, self.profiler_chan.clone()) {
            let tile_size = self.opts.tile_size;

            // FIXME: Try not to create a new array here.
            let mut new_buffers = ~[];

            // Divide up the layer into tiles.
            do time::profile(time::RenderingPrepBuffCategory, self.profiler_chan.clone()) {
                let mut y = 0;
                while y < (render_layer.size.height as f32 * scale).ceil() as uint {
                    let mut x = 0;
                    while x < (render_layer.size.width as f32 * scale).ceil() as uint {
                        // Figure out the dimension of this tile.
                        let right_max = (render_layer.size.width as f32 * scale).ceil() as uint;
                        let right = uint::min(x + tile_size, right_max);
                        let bottom_max = (render_layer.size.height as f32 * scale).ceil() as uint;
                        let bottom = uint::min(y + tile_size, bottom_max);
                        let width = right - x;
                        let height = bottom - y;

                        let tile_rect = Rect(Point2D(x as f32 / scale, y as f32 / scale),
                                             Size2D(width as f32, height as f32));
                        let screen_rect = Rect(Point2D(x, y), Size2D(width, height));

                        let size = Size2D(width as i32, height as i32);

                        // Make the current context current.
                        self.share_gl_context.make_current();

                        // Create the draw target.
                        let draw_target = if self.opts.gpu_rendering {
                            unsafe {
                                let native = self.share_gl_context.native();
                                DrawTarget::new_with_fbo(self.opts.render_backend,
                                                         cast::transmute(native),
                                                         size,
                                                         B8G8R8A8)
                            }
                        } else {
                            DrawTarget::new(self.opts.render_backend,
                                            size,
                                            B8G8R8A8)
                        };

                        // Create the layer buffer and an empty texture to use as a placeholder.
                        //
                        // FIXME(pcwalton): This is wasteful if GPU rendering is being used!
                        let mut buffer = LayerBuffer {
                            texture: Texture::new(),
                            rect: tile_rect,
                            screen_pos: screen_rect,
                            stride: (width * 4) as uint
                        };

                        {
                            // Build the render context.
                            let ctx = RenderContext {
                                canvas: &buffer,
                                draw_target: &draw_target,
                                font_ctx: self.font_ctx,
                                opts: &self.opts
                            };

                            // Apply the translation to render the tile we want.
                            let matrix: Matrix2D<AzFloat> = Matrix2D::identity();
                            let matrix = matrix.scale(scale as AzFloat, scale as AzFloat);
                            let matrix = matrix.translate(-(buffer.rect.origin.x) as AzFloat,
                                                          -(buffer.rect.origin.y) as AzFloat);

                            draw_target.set_transform(&matrix);

                            // Clear the buffer.
                            ctx.clear();

                            // Draw the display list.
                            do profile(RenderingDrawingCategory, self.profiler_chan.clone()) {
                                render_layer.display_list.draw_into_context(&ctx);
                                draw_target.flush();
                            }
                        }

                        if self.opts.gpu_rendering {
                            let texture_id = draw_target.steal_texture_id().get();
                            buffer.texture = Texture::adopt_native_texture(texture_id);
                        } else {
                            do draw_target.snapshot().get_data_surface().with_data |data| {
                                buffer.texture.upload_image(&TextureImageData {
                                    size: Size2D(width as uint, height as uint),
                                    stride: width,
                                    format: ARGB32Format,
                                    data: data,
                                })
                            }
                        }

                        new_buffers.push(buffer);

                        x += tile_size;
                    }

                    y += tile_size;
                }
            }

            let layer_buffer_set = LayerBufferSet {
                buffers: new_buffers,
            };

            debug!("renderer: returning surface");
            self.compositor.paint(layer_buffer_set, render_layer.size);
            self.compositor.set_render_state(IdleRenderState);
        }
    }
}

