/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::webgl_thread::{WebGLThread, WebGLThreadInit};
use canvas_traits::webgl::{WebGLContextId, WebGLMsg, WebGLSender, WebGLThreads};
use canvas_traits::webgl::{WebVRRenderHandler, webgl_channel};
use euclid::default::Size2D;
use fnv::FnvHashMap;
use gleam::gl;
use servo_config::pref;
use std::cell::RefCell;
use std::default::Default;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use surfman::{self, Context, Device, SurfaceTexture};
use swap_chains::SwapChains;
use webrender_traits::{WebrenderExternalImageApi, WebrenderExternalImageRegistry};
use webxr_api::WebGLExternalImageApi;

pub struct WebGLComm {
    pub webgl_threads: WebGLThreads,
    pub webxr_handler: Box<dyn webxr_api::WebGLExternalImageApi>,
    pub image_handler: Box<dyn WebrenderExternalImageApi>,
    pub output_handler: Option<Box<dyn webrender::OutputImageHandler>>,
}

impl WebGLComm {
    /// Creates a new `WebGLComm` object.
    pub fn new(
        device: Rc<Device>,
        context: Rc<RefCell<Context>>,
        webrender_gl: Rc<dyn gl::Gl>,
        webrender_api_sender: webrender_api::RenderApiSender,
        webvr_compositor: Option<Box<dyn WebVRRenderHandler>>,
        external_images: Arc<Mutex<WebrenderExternalImageRegistry>>,
        api_type: gl::GlType,
    ) -> WebGLComm {
        println!("WebGLThreads::new()");
        let (sender, receiver) = webgl_channel::<WebGLMsg>().unwrap();
        let webrender_swap_chains = SwapChains::new();

        // This implementation creates a single `WebGLThread` for all the pipelines.
        let init = WebGLThreadInit {
            webrender_api_sender,
            webvr_compositor,
            external_images,
            sender: sender.clone(),
            receiver,
            webrender_swap_chains: webrender_swap_chains.clone(),
            adapter: device.adapter(),
            api_type,
        };

        let output_handler = if pref!(dom.webgl.dom_to_texture.enabled) {
            Some(Box::new(OutputHandler::new(
                webrender_gl.clone(),
                sender.clone(),
            )))
        } else {
            None
        };

        let external = WebGLExternalImages::new(device,
                                                context,
                                                webrender_swap_chains);

        WebGLThread::run_on_own_thread(init);

        WebGLComm {
            webgl_threads: WebGLThreads(sender),
            webxr_handler: SendableWebGLExternalImages.clone_box(),
            image_handler: Box::new(external),
            output_handler: output_handler.map(|b| b as Box<_>),
        }
    }
}

/// Bridge between the webxr_api::ExternalImage callbacks and the WebGLThreads.
struct SendableWebGLExternalImages;

impl webxr_api::WebGLExternalImageApi for SendableWebGLExternalImages {
    fn lock(&self, _id: usize) -> Option<gl::GLsync> {
        // TODO(pcwalton)
        None
    }

    fn unlock(&self, _id: usize) {
        // TODO(pcwalton)
    }

    fn clone_box(&self) -> Box<dyn webxr_api::WebGLExternalImageApi> {
        Box::new(SendableWebGLExternalImages)
    }
}

/// Bridge between the webrender::ExternalImage callbacks and the WebGLThreads.
struct WebGLExternalImages {
    device: Rc<Device>,
    context: Rc<RefCell<Context>>,
    swap_chains: SwapChains<WebGLContextId>,
    locked_front_buffers: FnvHashMap<WebGLContextId, SurfaceTexture>,
}

impl WebGLExternalImages {
    fn new(device: Rc<Device>,
           context: Rc<RefCell<Context>>,
           swap_chains: SwapChains<WebGLContextId>)
           -> Self {
        Self {
            device,
            context,
            swap_chains,
            locked_front_buffers: FnvHashMap::default(),
        }
    }
}

impl WebGLExternalImages {
    fn lock_swap_chain(&mut self, id: WebGLContextId) -> Option<(u32, Size2D<i32>)> {
        println!("... locking chain {:?}", id);
        let front_buffer = self.swap_chains.get(id)?.take_surface()?;

        println!("... getting texture for surface {:?}", front_buffer.id());
        let mut context = self.context.borrow_mut();
        let size = front_buffer.size();
        let front_buffer_texture = self.device
                                       .create_surface_texture(&mut *context, front_buffer)
                                       .unwrap();
        let gl_texture = front_buffer_texture.gl_texture();

        self.locked_front_buffers.insert(id, front_buffer_texture);

        Some((gl_texture, size))
    }

    fn unlock_swap_chain(&mut self, id: WebGLContextId) -> Option<()> {
        let locked_front_buffer = self.locked_front_buffers.remove(&id)?;
        let mut context = self.context.borrow_mut();
        let locked_front_buffer = self.device
                                      .destroy_surface_texture(&mut *context, locked_front_buffer)
                                      .unwrap();

        println!("... unlocked chain {:?}", id);
        self.swap_chains.get(id)?.recycle_surface(locked_front_buffer);
        Some(())
    }
}

impl WebrenderExternalImageApi for WebGLExternalImages {
    fn lock(&mut self, id: u64) -> (u32, Size2D<i32>) {
        let id = WebGLContextId(id);
        self.lock_swap_chain(id).unwrap_or_default()
    }

    fn unlock(&mut self, id: u64) {
        let id = WebGLContextId(id);
        self.unlock_swap_chain(id);
    }
}

/// struct used to implement DOMToTexture feature and webrender::OutputImageHandler trait.
//type OutputHandlerData = Option<(u32, Size2D<i32>)>;
struct OutputHandler {
    webrender_gl: Rc<dyn gl::Gl>,
    webgl_channel: WebGLSender<WebGLMsg>,
    // Used to avoid creating a new channel on each received WebRender request.
    sync_objects: FnvHashMap<webrender_api::PipelineId, gl::GLsync>,
}

impl OutputHandler {
    fn new(webrender_gl: Rc<dyn gl::Gl>, channel: WebGLSender<WebGLMsg>) -> Self {
        OutputHandler {
            webrender_gl,
            webgl_channel: channel,
            sync_objects: Default::default(),
        }
    }
}

/// Bridge between the WR frame outputs and WebGL to implement DOMToTexture synchronization.
impl webrender::OutputImageHandler for OutputHandler {
    fn lock(
        &mut self,
        id: webrender_api::PipelineId,
    ) -> Option<(u32, webrender_api::units::FramebufferIntSize)> {
        // Insert a fence in the WR command queue
        let gl_sync = self
            .webrender_gl
            .fence_sync(gl::SYNC_GPU_COMMANDS_COMPLETE, 0);
        self.sync_objects.insert(id, gl_sync);
        None
    }

    fn unlock(&mut self, id: webrender_api::PipelineId) {
        if let Some(gl_sync) = self.sync_objects.remove(&id) {
            // Flush the Sync object into the GPU's command queue to guarantee that it it's signaled.
            self.webrender_gl.flush();
            // Mark the sync object for deletion.
            self.webrender_gl.delete_sync(gl_sync);
        }
    }
}
