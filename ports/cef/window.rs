/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Off-screen windows.
//!
//! This is used for off-screen rendering mode only; on-screen windows (the default embedding mode)
//! are managed by a platform toolkit (GLFW or Glutin).

use browser::CefBrowser;

use compositing::compositor_task::{mod, CompositorProxy, CompositorReceiver};
use compositing::windowing::{IdleWindowEvent, WindowEvent, WindowMethods};
use geom::scale_factor::ScaleFactor;
use geom::size::TypedSize2D;
use gleam::gl;
use layers::geometry::DevicePixel;
use layers::platform::surface::NativeGraphicsMetadata;
use libc::{c_char, c_void};
use servo_msg::compositor_msg::{ReadyState, RenderState};
use servo_util::geometry::ScreenPx;
use std::cell::RefCell;
use std::ptr;
use std::rc::Rc;

/// The type of an off-screen window.
#[deriving(Clone)]
pub struct Window {
    cef_browser: RefCell<Option<CefBrowser>>,
}

impl Window {
    /// Creates a new window.
    pub fn new() -> Rc<Window> {
        const RTLD_DEFAULT: *mut c_void = (-2) as *mut c_void;

        extern {
            fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
        }

        gl::load_with(|s| {
            unsafe {
                let c_str = s.to_c_str();
                dlsym(RTLD_DEFAULT, c_str.as_ptr()) as *const c_void
            }
        });

        Rc::new(Window {
            cef_browser: RefCell::new(None),
        })
    }

    /// Sets the current browser.
    pub fn set_browser(&self, browser: CefBrowser) {
        *self.cef_browser.borrow_mut() = Some(browser)
    }

    /// Currently unimplemented.
    pub fn wait_events(&self) -> WindowEvent {
        IdleWindowEvent
    }
}

impl WindowMethods for Window {
    fn framebuffer_size(&self) -> TypedSize2D<DevicePixel,uint> {
        let browser = self.cef_browser.borrow();
        match *browser {
            None => TypedSize2D(400, 300),
            Some(ref browser) => {
                let rect = browser.get_host()
                                  .get_client()
                                  .get_render_handler()
                                  .get_backing_rect(browser);
                TypedSize2D(rect.width as uint, rect.height as uint)
            }
        }
    }

    fn size(&self) -> TypedSize2D<ScreenPx,f32> {
        let browser = self.cef_browser.borrow();
        match *browser {
            None => TypedSize2D(400.0, 300.0),
            Some(ref browser) => {
                let rect = browser.get_host()
                                  .get_client()
                                  .get_render_handler()
                                  .get_view_rect(browser);
                TypedSize2D(rect.width as f32, rect.height as f32)
            }
        }
    }

    fn present(&self) {
        let browser = self.cef_browser.borrow();
        match *browser {
            None => {}
            Some(ref browser) => {
                browser.get_host().get_client().get_render_handler().present(browser.clone());
            }
        }
    }

    fn set_ready_state(&self, _: ReadyState) {
        // TODO(pcwalton)
    }

    fn set_render_state(&self, _: RenderState) {
        // TODO(pcwalton)
    }

    fn hidpi_factor(&self) -> ScaleFactor<ScreenPx,DevicePixel,f32> {
        let browser = self.cef_browser.borrow();
        match *browser {
            None => ScaleFactor(1.0),
            Some(ref browser) => {
                let view_rect = browser.get_host()
                                       .get_client()
                                       .get_render_handler()
                                       .get_view_rect(browser);
                let backing_rect = browser.get_host()
                                          .get_client()
                                          .get_render_handler()
                                          .get_backing_rect(browser);
                ScaleFactor(backing_rect.width as f32 / view_rect.width as f32)
            }
        }
    }

    #[cfg(target_os="macos")]
    fn native_metadata(&self) -> NativeGraphicsMetadata {
        use cgl::{CGLGetCurrentContext, CGLGetPixelFormat};

        // FIXME(pcwalton)
        unsafe {
            NativeGraphicsMetadata {
                pixel_format: CGLGetPixelFormat(CGLGetCurrentContext()),
            }
        }
    }

    fn create_compositor_channel(_: &Option<Rc<Window>>)
                                 -> (Box<CompositorProxy+Send>, Box<CompositorReceiver>) {
        let (sender, receiver) = channel();
        (box CefCompositorProxy {
             sender: sender,
         } as Box<CompositorProxy+Send>,
         box receiver as Box<CompositorReceiver>)
    }

    fn prepare_for_composite(&self) -> bool {
        let browser = self.cef_browser.borrow();
        match *browser {
            None => {}
            Some(ref browser) => {
                browser.get_host().get_client().get_render_handler().paint(browser.clone());
            }
        }
        true
    }

    fn url_changed(&self, _: &str) {
        // TODO(pcwalton)
    }
}

struct CefCompositorProxy {
    sender: Sender<compositor_task::Msg>,
}

impl CompositorProxy for CefCompositorProxy {
    #[cfg(target_os="macos")]
    fn send(&mut self, msg: compositor_task::Msg) {
        use cocoa::appkit::{NSApp, NSApplication, NSApplicationDefined, NSAutoreleasePool};
        use cocoa::appkit::{NSEvent, NSPoint};
        use cocoa::base::nil;

        // Send a message and kick the OS event loop awake.
        self.sender.send(msg);

        unsafe {
            let pool = NSAutoreleasePool::new(nil);
            let event =
                NSEvent::otherEventWithType_location_modifierFlags_timestamp_windowNumber_context_subtype_data1_data2(
                nil,
                NSApplicationDefined,
                NSPoint::new(0.0, 0.0),
                0,
                0.0,
                0,
                ptr::null_mut(),
                0,
                0,
                0);
            NSApp().postEvent_atStart_(event, false);
            pool.drain();
        }
    }

    fn clone_compositor_proxy(&self) -> Box<CompositorProxy+Send> {
        box CefCompositorProxy {
            sender: self.sender.clone(),
        } as Box<CompositorProxy+Send>
    }
}

