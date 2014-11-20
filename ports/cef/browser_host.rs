/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use browser::CefBrowser;
use client::CefClient;
use core;
use eutil;
use types::{KEYEVENT_CHAR, KEYEVENT_KEYDOWN, KEYEVENT_KEYUP, KEYEVENT_RAWKEYDOWN};
use types::{cef_browser_host_extra, cef_browser_host_t, cef_client_t, cef_key_event};
use types::{cef_mouse_button_type_t, cef_mouse_event};

use compositing::windowing::{InitializeCompositingWindowEvent, KeyEvent, MouseWindowClickEvent};
use compositing::windowing::{MouseWindowEventClass, MouseWindowMouseUpEvent, ResizeWindowEvent};
use compositing::windowing::{ScrollWindowEvent};
use geom::point::TypedPoint2D;
use geom::size::TypedSize2D;
use libc::c_int;
use servo_msg::constellation_msg::{mod, KeyModifiers, Pressed, Released, Repeated};
use std::mem;

def_cef_object!(CefBrowserHost, cef_browser_host_t)

impl CefBrowserHost {
    pub fn new(browser: CefBrowser, client: CefClient) -> CefBrowserHost {
        unsafe {
            let host = CefBrowserHost::from_c_object(
                eutil::create_cef_object::<cef_browser_host_t,cef_browser_host_extra>());
            (*host.c_object).get_client = Some(get_client);
            (*host.c_object).was_resized = Some(was_resized);
            (*host.c_object).send_mouse_click_event = Some(send_mouse_click_event);
            (*host.c_object).send_mouse_wheel_event = Some(send_mouse_wheel_event);
            (*host.c_object).send_key_event = Some(send_key_event);
            (*host.c_object).composite = Some(composite);
            (*host.c_object).initialize_compositing = Some(initialize_compositing);
            (*host.c_object).extra.browser = browser.c_object();
            (*host.c_object).extra.client = client.c_object();
            mem::forget(browser);
            mem::forget(client);
            host
        }
    }

    pub fn get_client(&self) -> CefClient {
        unsafe {
            CefClient::from_c_object_addref(((*self.c_object).get_client.unwrap())(self.c_object))
        }
    }
}

extern "C" fn get_client(this: *mut cef_browser_host_t) -> *mut cef_client_t {
    unsafe {
        let result = (*this).extra.client;
        eutil::add_ref(&mut (*result).base);
        result
    }
}

extern "C" fn was_resized(this: *mut cef_browser_host_t) {
    unsafe {
        let this = CefBrowserHost::from_c_object_addref(this);
        let browser = CefBrowser::from_c_object_addref((*this.c_object()).extra.browser);
        let rect = this.get_client().get_render_handler().get_backing_rect(&browser);
        core::send_window_event(ResizeWindowEvent(TypedSize2D(rect.width as uint,
                                                              rect.height as uint)));
    }
}

extern "C" fn send_mouse_click_event(_: *mut cef_browser_host_t,
                                     event: *const cef_mouse_event,
                                     mouse_button_type: cef_mouse_button_type_t,
                                     mouse_up: c_int,
                                     _: c_int) {
    unsafe {
        let button_type = mouse_button_type as uint;
        let point = TypedPoint2D((*event).x as f32, (*event).y as f32);
        if mouse_up != 0 {
            core::send_window_event(MouseWindowEventClass(MouseWindowClickEvent(button_type,
                                                                                point)))
        } else {
            core::send_window_event(MouseWindowEventClass(MouseWindowMouseUpEvent(button_type,
                                                                                  point)))
        }
    }
}

extern "C" fn send_mouse_wheel_event(_: *mut cef_browser_host_t,
                                     event: *const cef_mouse_event,
                                     delta_x: c_int,
                                     delta_y: c_int) {
    unsafe {
        let delta = TypedPoint2D(delta_x as f32, delta_y as f32);
        let origin = TypedPoint2D((*event).x as i32, (*event).y as i32);
        core::send_window_event(ScrollWindowEvent(delta, origin))
    }
}

extern "C" fn send_key_event(_: *mut cef_browser_host_t,
                             event: *const cef_key_event) {
    unsafe {
        // FIXME(pcwalton): So awful. But it's nearly midnight here and I have to get Google
        // working.
        let key = match (*event).character as u8 {
            b'a' | b'A' => constellation_msg::KeyA,
            b'b' | b'B' => constellation_msg::KeyB,
            b'c' | b'C' => constellation_msg::KeyC,
            b'd' | b'D' => constellation_msg::KeyD,
            b'e' | b'E' => constellation_msg::KeyE,
            b'f' | b'F' => constellation_msg::KeyF,
            b'g' | b'G' => constellation_msg::KeyG,
            b'h' | b'H' => constellation_msg::KeyH,
            b'i' | b'I' => constellation_msg::KeyI,
            b'j' | b'J' => constellation_msg::KeyJ,
            b'k' | b'K' => constellation_msg::KeyK,
            b'l' | b'L' => constellation_msg::KeyL,
            b'm' | b'M' => constellation_msg::KeyM,
            b'n' | b'N' => constellation_msg::KeyN,
            b'o' | b'O' => constellation_msg::KeyO,
            b'p' | b'P' => constellation_msg::KeyP,
            b'q' | b'Q' => constellation_msg::KeyQ,
            b'r' | b'R' => constellation_msg::KeyR,
            b's' | b'S' => constellation_msg::KeyS,
            b't' | b'T' => constellation_msg::KeyT,
            b'u' | b'U' => constellation_msg::KeyU,
            b'v' | b'V' => constellation_msg::KeyV,
            b'w' | b'W' => constellation_msg::KeyW,
            b'x' | b'X' => constellation_msg::KeyX,
            b'y' | b'Y' => constellation_msg::KeyY,
            b'z' | b'Z' => constellation_msg::KeyZ,
            b'0' => constellation_msg::Key0,
            b'1' => constellation_msg::Key1,
            b'2' => constellation_msg::Key2,
            b'3' => constellation_msg::Key3,
            b'4' => constellation_msg::Key4,
            b'5' => constellation_msg::Key5,
            b'6' => constellation_msg::Key6,
            b'7' => constellation_msg::Key7,
            b'8' => constellation_msg::Key8,
            b'9' => constellation_msg::Key9,
            b'\n' | b'\r' => constellation_msg::KeyEnter,
            _ => constellation_msg::KeySpace,
        };
        let key_state = match (*event).t {
            KEYEVENT_RAWKEYDOWN => Pressed,
            KEYEVENT_KEYDOWN | KEYEVENT_CHAR => Repeated,
            KEYEVENT_KEYUP => Released,
        };
        let key_modifiers = KeyModifiers::empty();  // TODO(pcwalton)
        core::send_window_event(KeyEvent(key, key_state, key_modifiers))
    }
}

extern "C" fn composite(_: *mut cef_browser_host_t) {
}

extern "C" fn initialize_compositing(_: *mut cef_browser_host_t) {
    println!("initializing compositing!");
    core::send_window_event(InitializeCompositingWindowEvent);
}

