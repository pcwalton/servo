/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use compositing::*;
use compositing::compositor_layer::CompositorLayer;
use platform::{Application, Window};
use windowing::{ApplicationMethods, WindowEvent, WindowMethods};
use windowing::{IdleWindowEvent, ResizeWindowEvent, LoadUrlWindowEvent, MouseWindowEventClass};
use windowing::{QuitWindowEvent, MouseWindowClickEvent, MouseWindowMouseDownEvent, MouseWindowMouseUpEvent};
use windowing::{ScrollWindowEvent, ZoomWindowEvent, NavigationWindowEvent, FinishedWindowEvent};

use azure::azure_hl::SourceSurfaceMethods;
use azure::azure_hl;
use extra::time::precise_time_s;
use geom::matrix::identity;
use geom::point::Point2D;
use geom::rect::Rect;
use geom::size::Size2D;
use layers::layers::{ContainerLayer, ContainerLayerKind};
use layers::platform::surface::NativeGraphicsMetadataDescriptor;
use layers::rendergl;
use layers::scene::Scene;
use opengles::gl2;
use png;
use servo_msg::compositor_msg::{CompositorComm, ChangeReadyState};
use servo_msg::compositor_msg::{ChangeRenderState, DeleteLayer, Exit, InvalidateRect, NewLayer};
use servo_msg::compositor_msg::{Paint, SetIds, SetLayerClipRect, SetLayerPageSize};
use servo_msg::constellation_msg::{FrameTreeReceivedMsg, InitLoadUrlMsg, LoadUrlMsg, NavigateMsg};
use servo_msg::constellation_msg::{ResizedWindowMsg, SetGraphicsMetadataMsg};
use servo_msg::constellation_msg;
use servo_util::time::profile;
use servo_util::{time, url};
use std::num::Orderable;
use std::path::Path;
use std::rt::io::timer::Timer;
use std::vec;

/// Starts the compositor, which listens for messages on the specified port.
pub fn run_compositor(compositor: &CompositorTask) {
    let app: Application = ApplicationMethods::new();
    let window: @mut Window = WindowMethods::new(&app);

    // Create an initial layer tree.
    //
    // TODO: There should be no initial layer tree until the renderer creates one from the display
    // list. This is only here because we don't have that logic in the renderer yet.
    let context = rendergl::init_render_context();
    let root_layer = @mut ContainerLayer();
    let window_size = window.size();
    let mut scene = Scene(ContainerLayerKind(root_layer), window_size, identity());
    let mut window_size = Size2D(window_size.width as uint, window_size.height as uint);
    let mut done = false;
    let mut recomposite = false;
    let graphics_context = CompositorTask::create_graphics_context();
    let mut comm = compositor.comm;

    // Keeps track of the current zoom factor
    let mut world_zoom = 1f32;
    let mut zoom_action = false;
    let mut zoom_time = 0f64;

    // The root CompositorLayer
    let mut compositor_layer: Option<CompositorLayer> = None;

    // Get BufferRequests from each layer.
    let ask_for_tiles = || {
        let window_size_page = Size2D(window_size.width as f32 / world_zoom,
                                      window_size.height as f32 / world_zoom);
        for layer in compositor_layer.mut_iter() {
            if !layer.hidden {
                let rect = Rect(Point2D(0f32, 0f32), window_size_page);
                let recomposite_result = layer.get_buffer_request(&graphics_context,
                                                                  &mut comm,
                                                                  rect,
                                                                  world_zoom);

                recomposite = recomposite_result || recomposite;
            } else {
                debug!("Compositor: root layer is hidden!");
            }
        }
    };

    let check_for_messages: &fn(&mut CompositorComm) = |comm: &mut CompositorComm| {
        // Handle messages
        while comm.peek() {
            match comm.recv() {
                Exit => done = true,

                ChangeReadyState(ready_state) => window.set_ready_state(ready_state),
                ChangeRenderState(render_state) => window.set_render_state(render_state),

                SetIds(frame_tree) => {
                    // This assumes there is at most one child, which should be the case.
                    match root_layer.first_child {
                        Some(old_layer) => root_layer.remove_child(old_layer),
                        None => {}
                    }

                    let layer = CompositorLayer::from_frame_tree(frame_tree,
                                                                 compositor.opts.tile_size,
                                                                 Some(10000000u),
                                                                 compositor.opts.cpu_painting);
                    root_layer.add_child_start(ContainerLayerKind(layer.root_layer));

                    // If there's already a root layer, destroy it cleanly.
                    match &mut compositor_layer {
                        &Some(ref mut compositor_layer) => compositor_layer.clear_all(comm),
                        _ => {}
                    }

                    compositor_layer = Some(layer);

                    // Initialize the new constellation channel by sending it the root window size.
                    //
                    // FIXME(pcwalton): This seems like not a great place to do this.
                    let window_size = window.size();
                    let window_size = Size2D(window_size.width as uint,
                                             window_size.height as uint);
                    comm.send(ResizedWindowMsg(window_size));
                    comm.send(FrameTreeReceivedMsg);
                }

                NewLayer(_id, new_size) => {
                    // FIXME: This should create an additional layer instead of replacing the current one.
                    // Once ResizeLayer messages are set up, we can switch to the new functionality.

                    let pipeline_id = match compositor_layer {
                        Some(ref compositor_layer) => compositor_layer.pipeline_id.clone(),
                        None => fail!("Compositor: Received new layer without initialized pipeline"),
                    };
                    let page_size = Size2D(new_size.width as f32, new_size.height as f32);
                    let new_layer = CompositorLayer::new(pipeline_id,
                                                         Some(page_size),
                                                         compositor.opts.tile_size,
                                                         Some(10000000u),
                                                         compositor.opts.cpu_painting);

                    let current_child = root_layer.first_child;
                    // This assumes there is at most one child, which should be the case.
                    match current_child {
                        Some(old_layer) => root_layer.remove_child(old_layer),
                        None => {}
                    }
                    root_layer.add_child_start(ContainerLayerKind(new_layer.root_layer));
                    compositor_layer = Some(new_layer);

                    ask_for_tiles();
                }

                SetLayerPageSize(id, new_size, epoch) => {
                    match compositor_layer {
                        Some(ref mut layer) => {
                            let page_window = Size2D(window_size.width as f32 / world_zoom,
                                                     window_size.height as f32 / world_zoom);
                            assert!(layer.resize(comm, id, new_size, page_window, epoch));
                            ask_for_tiles();
                        }
                        None => {}
                    }
                }

                SetLayerClipRect(id, new_rect) => {
                    match compositor_layer {
                        Some(ref mut layer) => {
                            assert!(layer.set_clipping_rect(id, new_rect));
                            ask_for_tiles();
                        }
                        None => {}
                    }
                }

                DeleteLayer(id) => {
                    match compositor_layer {
                        Some(ref mut layer) => {
                            assert!(layer.delete(&graphics_context, comm, id));
                            ask_for_tiles();
                        }
                        None => {}
                    }
                }

                Paint(id, new_layer_buffer_set, epoch) => {
                    debug!("osmain: received new frame");

                    // From now on, if we destroy the buffers, they will leak.
                    let mut new_layer_buffer_set = new_layer_buffer_set;
                    new_layer_buffer_set.mark_will_leak();

                    match compositor_layer {
                        Some(ref mut layer) => {
                            assert!(layer.add_buffers(&graphics_context,
                                                      comm,
                                                      id,
                                                      new_layer_buffer_set,
                                                      epoch).is_none());

                            recomposite = true;
                        }
                        None => {
                            fail!("Compositor: given paint command with no CompositorLayer initialized");
                        }
                    }
                    // TODO: Recycle the old buffers; send them back to the renderer to reuse if
                    // it wishes.
                }

                InvalidateRect(id, rect) => {
                    match compositor_layer {
                        Some(ref mut layer) => {
                            layer.invalidate_rect(id, Rect(Point2D(rect.origin.x as f32,
                                                                   rect.origin.y as f32),
                                                           Size2D(rect.size.width as f32,
                                                                  rect.size.height as f32)));
                            ask_for_tiles();
                        }
                        None => {} // Nothing to do
                    }
                }
            }
        }
    };

    let check_for_window_messages: &fn(WindowEvent) = |event| {
        match event {
            IdleWindowEvent => {}

            ResizeWindowEvent(width, height) => {
                let new_size = Size2D(width, height);
                if window_size != new_size {
                    debug!("osmain: window resized to {:u}x{:u}", width, height);
                    window_size = new_size;
                    comm.send(ResizedWindowMsg(new_size));
                } else {
                    debug!("osmain: dropping window resize since size is still {:u}x{:u}", width, height);
                }
            }

            LoadUrlWindowEvent(url_string) => {
                debug!("osmain: loading URL `{:s}`", url_string);
                let root_pipeline_id = match compositor_layer {
                    Some(ref layer) => layer.pipeline_id.clone(),
                    None => fail!("Compositor: Received LoadUrlWindowEvent without initialized compositor layers"),
                };
                comm.send(LoadUrlMsg(root_pipeline_id,
                                     url::make_url(url_string.to_str(), None).to_str(),
                                     window_size))
            }

            MouseWindowEventClass(mouse_window_event) => {
                let point = match mouse_window_event {
                    MouseWindowClickEvent(_, p) => Point2D(p.x / world_zoom, p.y / world_zoom),
                    MouseWindowMouseDownEvent(_, p) => Point2D(p.x / world_zoom, p.y / world_zoom),
                    MouseWindowMouseUpEvent(_, p) => Point2D(p.x / world_zoom, p.y / world_zoom),
                };

                // Send the event to the layer tree, which will forward it to the constellation.
                for layer in compositor_layer.iter() {
                    layer.send_mouse_event(&mut comm, mouse_window_event, point)
                }
            }

            ScrollWindowEvent(delta, cursor) => {
                // TODO: modify delta to snap scroll to pixels.
                let page_delta = Point2D(delta.x as f32 / world_zoom, delta.y as f32 / world_zoom);
                let page_cursor: Point2D<f32> = Point2D(cursor.x as f32 / world_zoom,
                                                        cursor.y as f32 / world_zoom);
                let page_window = Size2D(window_size.width as f32 / world_zoom,
                                         window_size.height as f32 / world_zoom);
                for layer in compositor_layer.mut_iter() {
                    recomposite = layer.scroll(page_delta, page_cursor, page_window) || recomposite;
                }
                ask_for_tiles();
            }

            ZoomWindowEvent(magnification) => {
                zoom_action = true;
                zoom_time = precise_time_s();
                let old_world_zoom = world_zoom;

                // Determine zoom amount
                world_zoom = (world_zoom * magnification).max(&1.0);
                root_layer.common.set_transform(identity().scale(world_zoom, world_zoom, 1f32));

                // Scroll as needed
                let page_delta = Point2D(window_size.width as f32 * (1.0 / world_zoom - 1.0 / old_world_zoom) * 0.5,
                                         window_size.height as f32 * (1.0 / world_zoom - 1.0 / old_world_zoom) * 0.5);
                // TODO: modify delta to snap scroll to pixels.
                let page_cursor = Point2D(-1f32, -1f32); // Make sure this hits the base layer
                let page_window = Size2D(window_size.width as f32 / world_zoom,
                                         window_size.height as f32 / world_zoom);
                for layer in compositor_layer.mut_iter() {
                    layer.scroll(page_delta, page_cursor, page_window);
                }

                recomposite = true;
            }

            NavigationWindowEvent(direction) => {
                let direction = match direction {
                    windowing::Forward => constellation_msg::Forward,
                    windowing::Back => constellation_msg::Back,
                };
                comm.send(NavigateMsg(direction))
            }

            FinishedWindowEvent => {
                if compositor.opts.exit_after_load {
                    done = true;
                }
            }

            QuitWindowEvent => {
                done = true;
            }
        }
    };


    let profiler_chan = compositor.profiler_chan.clone();
    let write_png = compositor.opts.output_file.is_some();
    let exit = compositor.opts.exit_after_load;
    let composite = || {
        do profile(time::CompositingCategory, profiler_chan.clone()) {
            debug!("compositor: compositing");
            // Adjust the layer dimensions as necessary to correspond to the size of the window.
            scene.size = window.size();

            // Render the scene.
            rendergl::render_scene(context, &scene);
        }

        // Render to PNG. We must read from the back buffer (ie, before
        // window.present()) as OpenGL ES 2 does not have glReadBuffer().
        if write_png {
            let (width, height) = (window_size.width as uint, window_size.height as uint);
            let path = from_str::<Path>(*compositor.opts.output_file.get_ref()).unwrap();
            let mut pixels = gl2::read_pixels(0, 0,
                                              width as gl2::GLsizei,
                                              height as gl2::GLsizei,
                                              gl2::RGB, gl2::UNSIGNED_BYTE);
            // flip image vertically (texture is upside down)
            let orig_pixels = pixels.clone();
            let stride = width * 3;
            for y in range(0, height) {
                let dst_start = y * stride;
                let src_start = (height - y - 1) * stride;
                vec::bytes::copy_memory(pixels.mut_slice(dst_start, dst_start + stride),
                                        orig_pixels.slice(src_start, src_start + stride),
                                        stride);
            }
            let img = png::Image {
                width: width as u32,
                height: height as u32,
                color_type: png::RGB8,
                pixels: pixels,
            };
            let res = png::store_png(&img, &path);
            assert!(res.is_ok());

            done = true;
        }

        window.present();

        if exit { done = true; }
    };

    // Send over the graphics metadata.
    let metadata_descriptor =
        NativeGraphicsMetadataDescriptor::from_metadata(azure_hl::current_graphics_metadata());
    comm.send(SetGraphicsMetadataMsg(metadata_descriptor));

    // Send over the initial URL(s).
    for url in compositor.opts.urls.iter() {
        comm.send(InitLoadUrlMsg(url::make_url(url.clone(), None).to_str()));
    }

    // Enter the main event loop.
    let mut tm = Timer::new().unwrap();
    while !done {
        // Check for new messages coming from the rendering task.
        check_for_messages(&mut comm);

        // Check for messages coming from the windowing system.
        check_for_window_messages(window.recv());

        if recomposite {
            recomposite = false;
            composite();
        }

        tm.sleep(10);

        // If a pinch-zoom happened recently, ask for tiles at the new resolution
        if zoom_action && precise_time_s() - zoom_time > 0.3 {
            zoom_action = false;
            ask_for_tiles();
        }

    }

    comm.send(constellation_msg::ExitMsg);

    // Clear out the compositor layers so that painting tasks can destroy the buffers.
    match compositor_layer {
        None => {}
        Some(ref mut layer) => layer.forget_all_tiles(),
    }
}
