/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::gl_context::GLContextFactory;
use crate::webgl_thread::{WebGLMainThread, WebGLThread, WebGLThreadInit};
use canvas_traits::webgl::webgl_channel;
use canvas_traits::webgl::DOMToTextureCommand;
use canvas_traits::webgl::{WebGLChan, WebGLContextId, WebGLLockMessage, WebGLMsg, WebGLPipeline};
use canvas_traits::webgl::{WebGLReceiver, WebGLSender, WebVRRenderHandler};
use embedder_traits::EventLoopWaker;
use euclid::default::Size2D;
use fnv::FnvHashMap;
use gleam::gl;
#[cfg(target_os = "macos")]
use io_surface;
use offscreen_gl_context::{NativeSurface, NativeSurfaceTexture};
use servo_config::pref;
use std::default::Default;
use std::mem;
use std::rc::Rc;
use std::sync::mpsc;
use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use webrender_traits::{WebrenderExternalImageApi, WebrenderExternalImageRegistry};
use webxr_api::WebGLExternalImageApi;

pub struct WebGLComm {
    pub webgl_threads: WebGLThreads,
    pub main_thread_data: Option<Rc<WebGLMainThread>>,
    pub webxr_handler: Box<dyn webxr_api::WebGLExternalImageApi>,
    pub image_handler: Box<dyn WebrenderExternalImageApi>,
    pub output_handler: Option<Box<dyn webrender::OutputImageHandler>>,
}

/// WebGL Threading API entry point that lives in the constellation.
pub struct WebGLThreads(WebGLSender<WebGLMsg>);

type IOSurfaceId = u32;

pub enum ThreadMode {
    MainThread(Box<dyn EventLoopWaker>),
    OffThread,
}

impl WebGLComm {
    /// Creates a new `WebGLComm` object.
    pub fn new(
        gl_factory: GLContextFactory,
        webrender_gl: Rc<dyn gl::Gl>,
        webrender_api_sender: webrender_api::RenderApiSender,
        webvr_compositor: Option<Box<dyn WebVRRenderHandler>>,
        external_images: Arc<Mutex<WebrenderExternalImageRegistry>>,
        mode: ThreadMode,
    ) -> WebGLComm {
        println!("WebGLThreads::new()");
        let (sender, receiver) = webgl_channel::<WebGLMsg>().unwrap();

        // Make a front buffer.
        let front_buffer = Arc::new(FrontBuffer::new());

        // This implementation creates a single `WebGLThread` for all the pipelines.
        let init = WebGLThreadInit {
            gl_factory,
            webrender_api_sender,
            webvr_compositor,
            external_images,
            sender: sender.clone(),
            receiver,
            front_buffer: front_buffer.clone(),
        };

        let output_handler = if pref!(dom.webgl.dom_to_texture.enabled) {
            Some(Box::new(OutputHandler::new(
                webrender_gl.clone(),
                sender.clone(),
            )))
        } else {
            None
        };

        let external = WebGLExternalImages::new(webrender_gl, front_buffer, sender.clone());

        let webgl_thread = match mode {
            ThreadMode::MainThread(event_loop_waker) => {
                let thread = WebGLThread::run_on_current_thread(init, event_loop_waker);
                Some(thread)
            },
            ThreadMode::OffThread => {
                WebGLThread::run_on_own_thread(init);
                None
            },
        };

        WebGLComm {
            webgl_threads: WebGLThreads(sender),
            main_thread_data: webgl_thread,
            webxr_handler: external.sendable.clone_box(),
            image_handler: Box::new(external),
            output_handler: output_handler.map(|b| b as Box<_>),
        }
    }
}

impl WebGLThreads {
    /// Gets the WebGLThread handle for each script pipeline.
    pub fn pipeline(&self) -> WebGLPipeline {
        // This mode creates a single thread, so the existing WebGLChan is just cloned.
        WebGLPipeline(WebGLChan(self.0.clone()))
    }

    /// Sends a exit message to close the WebGLThreads and release all WebGLContexts.
    pub fn exit(&self) -> Result<(), &'static str> {
        self.0
            .send(WebGLMsg::Exit)
            .map_err(|_| "Failed to send Exit message")
    }
}

/// Bridge between the webxr_api::ExternalImage callbacks and the WebGLThreads.
struct SendableWebGLExternalImages {
    webgl_channel: WebGLSender<WebGLMsg>,
    // Used to avoid creating a new channel on each received WebRender request.
    lock_channel: (
        WebGLSender<WebGLLockMessage>,
        WebGLReceiver<WebGLLockMessage>,
    ),
}

impl SendableWebGLExternalImages {
    fn new(channel: WebGLSender<WebGLMsg>) -> Self {
        Self {
            webgl_channel: channel,
            lock_channel: webgl_channel().unwrap(),
        }
    }
}

impl webxr_api::WebGLExternalImageApi for SendableWebGLExternalImages {
    fn lock(&self, id: usize) -> Option<gl::GLsync> {
        // TODO(pcwalton)
        None
    }

    fn unlock(&self, id: usize) {
        // TODO(pcwalton)
    }

    fn clone_box(&self) -> Box<dyn webxr_api::WebGLExternalImageApi> {
        Box::new(Self::new(self.webgl_channel.clone()))
    }
}

/// Bridge between the webrender::ExternalImage callbacks and the WebGLThreads.
struct WebGLExternalImages {
    webrender_gl: Rc<dyn gl::Gl>,
    front_buffer: Arc<FrontBuffer>,
    locked_front_buffer: Option<NativeSurfaceTexture>,
    sendable: SendableWebGLExternalImages,
}

impl WebGLExternalImages {
    fn new(webrender_gl: Rc<dyn gl::Gl>,    
           front_buffer: Arc<FrontBuffer>,
           channel: WebGLSender<WebGLMsg>)
           -> Self {
        Self {
            webrender_gl,
            front_buffer,
            locked_front_buffer: None,
            sendable: SendableWebGLExternalImages::new(channel),
        }
    }
}

impl WebrenderExternalImageApi for WebGLExternalImages {
    fn lock(&mut self, id: u64) -> (u32, Size2D<i32>) {
        /*
        // WebGL Thread has it's own GL command queue that we need to synchronize with the WR GL
        // command queue. The WebGLMsg::Lock message inserts a fence in the WebGL command queue.
        self.sendable
            .webgl_channel
            .send(WebGLMsg::Lock(
                WebGLContextId(id as usize),
                self.surface_texture.take().map(|surface_texture| {
                    surface_texture.into_surface(&*self.webrender_gl)
                }),
                self.sendable.lock_channel.0.clone(),
            ))
            .unwrap();

        // Take the surface from WebGL and wrap it in a native surface texture.
        let WebGLLockMessage { mut surface, sync } = self.sendable.lock_channel.1.recv().unwrap();
        self.surface_texture = Some(NativeSurfaceTexture::new(&*self.webrender_gl, surface));
        */

        let (gl_texture, size);
        match self.front_buffer.take() {
            None => {
                gl_texture = 0;
                size = Size2D::new(0, 0);
                self.locked_front_buffer = None;
            }
            Some(front_buffer) => {
                let locked_front_buffer = NativeSurfaceTexture::new(&*self.webrender_gl,
                                                                    front_buffer);
                gl_texture = locked_front_buffer.gl_texture();
                size = locked_front_buffer.surface().size();
                println!("(lock) presenting surface {}", locked_front_buffer.surface().id());
                self.locked_front_buffer = Some(locked_front_buffer);
            }
        }

        println!("(lock) returning gl texture: {:?}", gl_texture);
        (gl_texture, size)
    }

    fn unlock(&mut self, id: u64) {
        self.sendable.unlock(id as usize);
        if let Some(locked_front_buffer) = self.locked_front_buffer.take() {
            println!("(unlock) putting back surface {}", locked_front_buffer.surface().id());
            self.front_buffer.put_back(locked_front_buffer.into_surface(&*self.webrender_gl));
        }
    }
}

pub struct FrontBuffer(Mutex<Option<NativeSurface>>);

impl FrontBuffer {
    fn new() -> FrontBuffer {
        FrontBuffer(Mutex::new(None))
    }

    fn take(&self) -> Option<NativeSurface> {
        self.0.lock().unwrap().take()
    }

    fn put_back(&self, old_front_buffer: NativeSurface) {
        let mut slot = self.0.lock().unwrap();
        if slot.is_none() {
            *slot = Some(old_front_buffer);
        } else {
            println!("*** (unlock) front buffer already has surface {}, dropping",
                     slot.as_ref().unwrap().id());
        }
    }

    pub(crate) fn lock(&self) -> MutexGuard<Option<NativeSurface>> {
        self.0.lock().unwrap()
    }
}

/// struct used to implement DOMToTexture feature and webrender::OutputImageHandler trait.
type OutputHandlerData = Option<(u32, Size2D<i32>)>;
struct OutputHandler {
    webrender_gl: Rc<dyn gl::Gl>,
    webgl_channel: WebGLSender<WebGLMsg>,
    // Used to avoid creating a new channel on each received WebRender request.
    lock_channel: (
        WebGLSender<OutputHandlerData>,
        WebGLReceiver<OutputHandlerData>,
    ),
    sync_objects: FnvHashMap<webrender_api::PipelineId, gl::GLsync>,
}

impl OutputHandler {
    fn new(webrender_gl: Rc<dyn gl::Gl>, channel: WebGLSender<WebGLMsg>) -> Self {
        OutputHandler {
            webrender_gl,
            webgl_channel: channel,
            lock_channel: webgl_channel().unwrap(),
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

        let result = if let Some(main_thread) = WebGLMainThread::on_current_thread() {
            main_thread
                .thread_data
                .borrow_mut()
                .handle_dom_to_texture_lock(id, gl_sync as usize)
        } else {
            // The lock command adds a WaitSync call on the WebGL command flow.
            let command =
                DOMToTextureCommand::Lock(id, gl_sync as usize, self.lock_channel.0.clone());
            self.webgl_channel
                .send(WebGLMsg::DOMToTextureCommand(command))
                .unwrap();
            self.lock_channel.1.recv().unwrap()
        };

        result.map(|(tex_id, size)| {
            (
                tex_id,
                webrender_api::units::FramebufferIntSize::new(size.width, size.height),
            )
        })
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
