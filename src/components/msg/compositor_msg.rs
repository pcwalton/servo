/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use constellation_msg::PipelineId;
use constellation_msg;

use extra::serialize::{Encodable, Encoder};
use geom::rect::Rect;
use geom::size::Size2D;
use layers::platform::surface::{NativePaintingGraphicsContext, NativeSurface};
use layers::platform::surface::{NativeSurfaceMethods};
use util::ipc::{MsgReader, MsgWriter, NativeUnixStream};

/// The compositor's end of the compositor-to-constellation communications channel.
pub struct CompositorComm {
    /// The native Unix domain server.
    priv stream: NativeUnixStream,
}

impl CompositorComm {
    /// Sets up the compositor communication by opening a Unix domain socket.
    ///
    /// Be warned: This uses *synchronous I/O* to open the local socket, because we don't have the
    /// Rust runtime here.
    #[fixed_stack_segment]
    pub fn init(stream: NativeUnixStream) -> CompositorComm {
        CompositorComm {
            stream: stream,
        }
    }

    /// Returns true if a message is waiting on the port and false otherwise.
    #[fixed_stack_segment]
    pub fn peek(&self) -> bool {
        self.stream.peek()
    }

    /// Receives a message from the constellation.
    #[fixed_stack_segment]
    pub fn recv(&mut self) -> Msg {
        self.stream.read_msg()
    }

    /// Encodes and sends a message to the constellation.
    pub fn send(&mut self, msg: constellation_msg::Msg) {
        self.stream.write_msg(msg)
    }
}

#[deriving(Decodable, Encodable)]
pub struct LayerBuffer {
    /// The native surface which can be shared between threads or processes. On Mac this is an
    /// `IOSurface`; on Linux this is an X Pixmap; on Android this is an `EGLImageKHR`.
    native_surface: NativeSurface,

    /// The rect in the containing RenderLayer that this represents.
    rect: Rect<f32>,

    /// The rect in pixels that will be drawn to the screen.
    screen_pos: Rect<uint>,

    /// The scale at which this tile is rendered
    resolution: f32,

    /// NB: stride is in pixels, like OpenGL GL_UNPACK_ROW_LENGTH.
    stride: uint,
}

/// A set of layer buffers. This is an atomic unit used to switch between the front and back
/// buffers.
#[deriving(Decodable, Encodable)]
pub struct LayerBufferSet {
    buffers: ~[~LayerBuffer]
}

impl LayerBufferSet {
    /// Notes all buffer surfaces will leak if not destroyed via a call to `destroy`.
    pub fn mark_will_leak(&mut self) {
        for buffer in self.buffers.mut_iter() {
            buffer.native_surface.mark_will_leak()
        }
    }
}

/// The status of the renderer.
#[deriving(Decodable, Encodable, Eq)]
pub enum RenderState {
    IdleRenderState,
    RenderingRenderState,
}

#[deriving(Decodable, Encodable, Eq)]
pub enum ReadyState {
    /// Informs the compositor that nothing has been done yet. Used for setting status
    Blank,
    /// Informs the compositor that a page is loading. Used for setting status
    Loading,
    /// Informs the compositor that a page is performing layout. Used for setting status
    PerformingLayout,
    /// Informs the compositor that a page is finished loading. Used for setting status
    FinishedLoading,
}

/// A newtype struct for denoting the age of messages; prevents race conditions.
#[deriving(Decodable, Encodable, Eq)]
pub struct Epoch(uint);

impl Epoch {
    pub fn next(&mut self) {
        **self += 1;
    }
}

/// The interface used by the quadtree and buffer map to get info about layer buffers.
pub trait Tile {
    /// Returns the amount of memory used by the tile
    fn get_mem(&self) -> uint;

    /// Returns true if the tile is displayable at the given scale
    fn is_valid(&self, f32) -> bool;

    /// Returns the size of the tile.
    fn get_size_2d(&self) -> Size2D<uint>;

    /// Marks the layer buffer as not leaking. See comments on
    /// `NativeSurfaceMethods::mark_wont_leak` for how this is used.
    fn mark_wont_leak(&mut self);

    /// Destroys the layer buffer. Painting task only.
    fn destroy(self, graphics_context: &NativePaintingGraphicsContext);
}

impl Tile for ~LayerBuffer {
    fn get_mem(&self) -> uint {
        // This works for now, but in the future we may want a better heuristic
        self.screen_pos.size.width * self.screen_pos.size.height
    }
    fn is_valid(&self, scale: f32) -> bool {
        self.resolution.approx_eq(&scale)
    }
    fn get_size_2d(&self) -> Size2D<uint> {
        self.screen_pos.size
    }
    fn mark_wont_leak(&mut self) {
        self.native_surface.mark_wont_leak()
    }
    fn destroy(self, graphics_context: &NativePaintingGraphicsContext) {
        let mut this = self;
        this.native_surface.destroy(graphics_context)
    }
}

/// Messages from the painting task and the constellation task to the compositor task.
#[deriving(Decodable, Encodable)]
pub enum Msg {
    /// Requests that the compositor shut down.
    Exit,

    /// Alerts the compositor that there is a new layer to be rendered.
    NewLayer(PipelineId, Size2D<f32>),
    /// Alerts the compositor that the specified layer's page has changed size.
    SetLayerPageSize(PipelineId, Size2D<f32>, Epoch),
    /// Alerts the compositor that the specified layer's clipping rect has changed.
    SetLayerClipRect(PipelineId, Rect<f32>),
    /// Alerts the compositor that the specified layer has been deleted.
    DeleteLayer(PipelineId),
    /// Invalidate a rect for a given layer
    InvalidateRect(PipelineId, Rect<uint>),

    /// Requests that the compositor paint the given layer buffer set for the given page size.
    Paint(PipelineId, ~LayerBufferSet, Epoch),
    /// Alerts the compositor to the current status of page loading.
    ChangeReadyState(ReadyState),
    /// Alerts the compositor to the current status of rendering.
    ChangeRenderState(RenderState),
    /// Sets the channel to the current layout and render tasks, along with their id
    SetIds(SendableFrameTree),
}

#[deriving(Decodable, Encodable)]
pub struct SendableFrameTree {
    pipeline_id: PipelineId,
    children: ~[SendableChildFrameTree],
}

#[deriving(Decodable, Encodable)]
pub struct SendableChildFrameTree {
    frame_tree: SendableFrameTree,
    rect: Option<Rect<f32>>,
}

impl SendableFrameTree {
    fn contains(&self, id: PipelineId) -> bool {
        if self.pipeline_id == id {
            return true
        }
        do self.children.iter().any |&SendableChildFrameTree { frame_tree: ref frame_tree, _ }| {
            frame_tree.contains(id)
        }
    }
}

