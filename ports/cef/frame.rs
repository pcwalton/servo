/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use core;
use eutil;
use string::CefStringRef;
use types::{cef_frame_extra, cef_frame_t, cef_string_t, cef_string_userfree_t};

use compositing::windowing::LoadUrlWindowEvent;

pub fn create_cef_frame() -> *mut cef_frame_t {
    unsafe {
        let frame = eutil::create_cef_object::<cef_frame_t,cef_frame_extra>();
        (*frame).load_url = load_url;
        (*frame).get_url = get_url;
        frame
    }
}

extern "C" fn load_url(_frame: *mut cef_frame_t, url: *mut cef_string_t) {
    unsafe {
        let url = CefStringRef::from_c_object(&url).to_string();
        println!("loading URL: {}!", url);
        core::send_window_event(LoadUrlWindowEvent(url));
    }
}

extern "C" fn get_url(_frame: *mut cef_frame_t) -> cef_string_userfree_t {
    // TODO(pcwalton)
    panic!("TODO")
}

