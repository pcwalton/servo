/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use interfaces::{CefFrame, cef_frame_t};
use string::CefStringRef;
use types::{cef_string_t, cef_string_userfree_t};

use core;
use compositing::windowing::LoadUrlWindowEvent;

pub struct ServoCefFrame;

cef_class_impl! {
    ServoCefFrame : CefFrame, cef_frame_t {
        fn load_url(&_this, url: *const cef_string_t) -> () {
            unsafe {
                let url = CefStringRef::from_c_object(&url).to_string();
                core::send_window_event(LoadUrlWindowEvent(url));
            }
        }
        fn get_url(&_this) -> cef_string_userfree_t {
            match core::url_for_main_frame() {
                None => "".to_string(),
                Some(url) => url.to_string(),
            }
        }
    }
}

