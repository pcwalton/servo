/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The high-level interface from script to constellation. Using this abstract interface helps reduce
/// coupling between these two components

use compositor_msg::{ChangeReadyState, ChangeRenderState, DeleteLayer, Epoch};
use compositor_msg::{Exit, InvalidateRect, LayerBuffer, LayerBufferSet, NewLayer, Paint};
use compositor_msg::{ReadyState, RenderState, SetLayerClipRect, SetLayerPageSize};
use compositor_msg;

use extra::serialize::{Decoder, Encodable, Encoder};
use geom::point::Point2D;
use geom::size::Size2D;
use geom::rect::Rect;
use layers::platform::surface::NativeGraphicsMetadataDescriptor;
use std::comm::{SharedChan, SharedPort};
use util::ipc::{MsgReader, MsgWriter, NativeUnixStream};

/// The interface used by the renderer to acquire draw targets for each render frame and
/// submit them to be drawn to the display.
pub trait RenderListener {
    /// Creates a new layer of the given size in the given pipeline ID.
    fn new_layer(&mut self, PipelineId, Size2D<uint>);

    /// Sets the size of the scrollable area of the given layer.
    fn set_layer_page_size(&mut self, PipelineId, Size2D<uint>, Epoch);

    /// Sets the clipping bounds of the given layer.
    fn set_layer_clip_rect(&mut self, PipelineId, Rect<uint>);

    /// Deletes a layer.
    fn delete_layer(&mut self, PipelineId);

    /// Sends new layer contents. The `Epoch` is used for time synchronization.
    fn paint(&mut self, id: PipelineId, layer_buffer_set: ~LayerBufferSet, Epoch);

    /// Sets the render state (idle or rendering). This is used for progress messages.
    fn set_render_state(&mut self, render_state: RenderState);
}

/// The interface used by the script task to tell the compositor to update its ready state,
/// which is used in displaying the appropriate message in the window's title.
pub trait ScriptListener : Clone {
    /// Sets the ready state, for progress messages.
    fn set_ready_state(&mut self, ReadyState);
    fn invalidate_rect(&mut self, PipelineId, Rect<uint>);
    fn close(&mut self);
}

/// Encapsulates the IPC messages from the constellation to the compositor.
#[deriving(Clone)]
pub struct ConstellationStream {
    /// The IPC connection to the compositor.
    priv stream: NativeUnixStream,
}

impl ConstellationStream {
    /// Creates a new constellation messenger. This is expected to be called in the task in which
    /// the constellation manager runs.
    pub fn init(stream: NativeUnixStream) -> ConstellationStream {
        ConstellationStream {
            stream: stream,
        }
    }

    /// Closes the stream.
    pub fn close(&self) {
        self.stream.close()
    }

    /// Receives a message from the compositor.
    pub fn recv(&mut self) -> Msg {
        self.stream.read_msg()
    }

    /// Sends a message to the compositor.
    pub fn send(&mut self, msg: compositor_msg::Msg) {
        self.stream.write_msg(msg)
    }
}


/// The constellation's endpoint of the compositor-to-constellation message channel.
#[deriving(Clone)]
pub struct ConstellationComm {
    /// The port on which constellation messages are received. Messages from the compositor go
    /// through this port.
    port: SharedPort<Msg>,

    /// A channel we can clone to allow others within this process to send messages to us.
    constellation_chan: SharedChan<Msg>,

    /// The IPC channel to the compositor.
    constellation_stream: ConstellationStream,
}

impl ConstellationComm {
    /// Connects to the compositor over a Unix socket and returns the constellation's communication
    /// object.
    pub fn init(port: Port<Msg>,
                constellation_chan: SharedChan<Msg>,
                constellation_stream: ConstellationStream)
                -> ConstellationComm {
        ConstellationComm {
            port: SharedPort::new(port),
            constellation_chan: constellation_chan,
            constellation_stream: constellation_stream,
        }
    }

    pub fn get_constellation_chan(&self) -> SharedChan<Msg> {
        self.constellation_chan.clone()
    }

    /// Sends a message to the compositor.
    pub fn send(&mut self, msg: compositor_msg::Msg) {
        self.constellation_stream.send(msg);
    }

    /// Receives a message from the compositor or other tasks.
    pub fn recv(&mut self) -> Msg {
        self.port.recv()
    }
}

/// Implementation of the abstract `ScriptListener` interface.
impl ScriptListener for ConstellationComm {
    fn set_ready_state(&mut self, ready_state: ReadyState) {
        let msg = ChangeReadyState(ready_state);
        self.send(msg);
    }

    fn invalidate_rect(&mut self, id: PipelineId, rect: Rect<uint>) {
        self.send(InvalidateRect(id, rect));
    }

    fn close(&mut self) {
        self.send(Exit);
    }
}

/// Implementation of the abstract `RenderListener` interface.
impl RenderListener for ConstellationComm {
    fn paint(&mut self, id: PipelineId, layer_buffer_set: ~LayerBufferSet, epoch: Epoch) {
        self.send(Paint(id, layer_buffer_set, epoch))
    }

    fn new_layer(&mut self, id: PipelineId, page_size: Size2D<uint>) {
        let Size2D { width, height } = page_size;
        self.send(NewLayer(id, Size2D(width as f32, height as f32)))
    }
    fn set_layer_page_size(&mut self, id: PipelineId, page_size: Size2D<uint>, epoch: Epoch) {
        let Size2D { width, height } = page_size;
        self.send(SetLayerPageSize(id, Size2D(width as f32, height as f32), epoch))
    }
    fn set_layer_clip_rect(&mut self, id: PipelineId, new_rect: Rect<uint>) {
        let new_rect = Rect(Point2D(new_rect.origin.x as f32,
                                    new_rect.origin.y as f32),
                            Size2D(new_rect.size.width as f32,
                                   new_rect.size.height as f32));
        self.send(SetLayerClipRect(id, new_rect))
    }

    fn delete_layer(&mut self, id: PipelineId) {
        self.send(DeleteLayer(id))
    }

    fn set_render_state(&mut self, render_state: RenderState) {
        self.send(ChangeRenderState(render_state))
    }
}

#[deriving(Decodable, Encodable, Eq)]
pub enum IFrameSandboxState {
    IFrameSandboxed,
    IFrameUnsandboxed
}

/// Events that the compositor can send to script.
#[deriving(Decodable, Encodable)]
pub enum ScriptEvent {
    ResizeEvent(uint, uint), 
    ReflowEvent,
    ClickEvent(uint, Point2D<f32>),
    MouseDownEvent(uint, Point2D<f32>),
    MouseUpEvent(uint, Point2D<f32>),
}

/// Messages from the compositor to the constellation.
///
/// FIXME(pcwalton): Many of these `~str`s are actually URLs, which are sadly not encodable and
/// decodable. :(
#[deriving(Decodable, Encodable)]
pub enum Msg {
    /// Requests that the constellation shut down.
    ExitMsg,

    FailureMsg(PipelineId, Option<SubpageId>),
    InitLoadUrlMsg(~str),

    /// Indicates that a particular `iframe` has changed size. The `Rect` parameter indicates the
    /// new size in page coordinates.
    FrameRectMsg(PipelineId, SubpageId, Rect<f32>),

    /// Sent to acknowledge that a new frame tree was received.
    FrameTreeReceivedMsg,

    /// Indicates that a new URL has been loaded on the outermost page.
    LoadUrlMsg(PipelineId, ~str, Size2D<uint>),

    /// Indicates that a new `iframe` was encountered. At the moment this is encountered, the
    /// `iframe`'s size is not yet known. Therefore, the compositor will queue this message but
    /// will not act on it until the appropriate `FrameRectMsg` arrives to establish its size.
    LoadIframeUrlMsg(~str, PipelineId, SubpageId, IFrameSandboxState),

    NavigateMsg(NavigationDirection),

    RendererReadyMsg(PipelineId),
    ResizedWindowMsg(Size2D<uint>),

    // The following messages are "forwarding messages": they do nothing but forward to either the
    // script task or render task of the given pipeline.
    //
    // TODO(pcwalton): Maybe refactor this a bit so we don't duplicate the arguments?

    /// Sends a request to the painting task to paint tiles at the given positions.
    ReRenderMsg(PipelineId, ~[BufferRequest], f32, Epoch),

    /// Sends an event to the given pipeline.
    SendEventMsg(PipelineId, ScriptEvent),

    /// Sets the graphics metadata. This must be sent before loading any pages if any rendering is
    /// to be performed, as the render task cannot render anything without the graphics metadata.
    /// It is still possible to load pages without the graphics metadata, but Servo will
    /// effectively be in "headless" mode then: no graphics output will be displayed.
    ///
    /// On Mac, this is the pixel format; on Linux, this is the X server `DISPLAY` variable.
    SetGraphicsMetadataMsg(NativeGraphicsMetadataDescriptor),

    /// Sends back now-unused layer buffers to the given pipeline to be recycled.
    UnusedBufferMsg(PipelineId, ~[~LayerBuffer]),
}

/// A request from the compositor to the renderer for tiles that need to be (re)displayed.
#[deriving(Clone, Decodable, Encodable)]
pub struct BufferRequest {
    // The rect in pixels that will be drawn to the screen
    screen_rect: Rect<uint>,
    
    // The rect in page coordinates that this tile represents
    page_rect: Rect<f32>,
}

impl BufferRequest {
    pub fn init(screen_rect: Rect<uint>, page_rect: Rect<f32>) -> BufferRequest {
        BufferRequest {
            screen_rect: screen_rect,
            page_rect: page_rect,
        }
    }
}

/// Represents the two different ways to which a page can be navigated
#[deriving(Clone, Eq, IterBytes)]
pub enum NavigationType {
    Load,               // entered or clicked on a url
    Navigate,           // browser forward/back buttons
}

#[deriving(Clone, Decodable, Encodable, Eq, IterBytes)]
pub enum NavigationDirection {
    Forward,
    Back,
}

/// A value that uniquely identifies a pipeline.
#[deriving(Clone, Decodable, Encodable, Eq, IterBytes)]
pub struct PipelineId(uint);

/// A value that uniquely identifies a subpage (`iframe`, usually) within a pipeline.
#[deriving(Clone, Decodable, Encodable, Eq, IterBytes)]
pub struct SubpageId(uint);

