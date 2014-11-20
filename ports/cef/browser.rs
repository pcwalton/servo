/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use browser_host::CefBrowserHost;
use client::CefClient;
use core::{mod, OffScreenGlobals, OnScreenGlobals, globals};
use eutil;
use frame;
use servo::Browser;
use types::{cef_browser_settings_t, cef_browser_extra, cef_browser_host_t, cef_browser_t};
use types::{cef_client_t, cef_frame_t, cef_request_context_t, cef_string_t, cef_window_info_t};
use window;

use compositing::windowing::{Back, Forward, NavigationWindowEvent};
use glfw_app;
use libc::c_int;
use std::cell::RefCell;
use std::mem;

def_cef_object!(CefBrowser, cef_browser_t)

impl CefBrowser {
    pub fn get_host(&self) -> CefBrowserHost {
        unsafe {
            CefBrowserHost::from_c_object_addref(((*self.c_object).get_host.unwrap())(
                    self.c_object))
        }
    }
}

#[no_mangle]
pub extern "C" fn cef_browser_host_create_browser(_window_info: *const cef_window_info_t,
                                                  _client: *mut cef_client_t,
                                                  _url: *const cef_string_t,
                                                  _settings: *const cef_browser_settings_t,
                                                  _request_context: *mut cef_request_context_t)
                                                  -> c_int {
    0
}

#[no_mangle]
pub extern "C" fn cef_browser_host_create_browser_sync(window_info: *const cef_window_info_t,
                                                       client: *mut cef_client_t,
                                                       _url: *const cef_string_t,
                                                       _settings: *const cef_browser_settings_t,
                                                       _request_context: *mut cef_request_context_t)
                                                       -> *mut cef_browser_t {

    unsafe {
        let browser = CefBrowser::from_c_object(eutil::create_cef_object::<cef_browser_t,
                                                                           cef_browser_extra>());
        (*browser.c_object).go_back = Some(go_back);
        (*browser.c_object).go_forward = Some(go_forward);
        (*browser.c_object).get_main_frame = Some(get_main_frame);
        (*browser.c_object).get_host = Some(get_host);
        (*browser.c_object).extra.frame = frame::create_cef_frame();

        let host = CefBrowserHost::new(browser.clone(), CefClient::from_c_object(client));
        (*browser.c_object).extra.host = host.c_object();
        mem::forget(host);

        if (*window_info).windowless_rendering_enabled == 0 {
            let glfw_window = glfw_app::create_window();
            globals.replace(Some(OnScreenGlobals(RefCell::new(glfw_window.clone()),
                                                 RefCell::new(Browser::new(Some(glfw_window))))));
        } else {
            let window = window::Window::new();
            let servo_browser = Browser::new(Some(window.clone()));
            window.set_browser(browser.clone());
            globals.replace(Some(OffScreenGlobals(RefCell::new(window),
                                                  RefCell::new(servo_browser))));
        }

        let c_object = browser.c_object();
        mem::forget(browser);
        c_object
    }
}

extern "C" fn go_back(_: *mut cef_browser_t) {
    core::send_window_event(NavigationWindowEvent(Back));
}

extern "C" fn go_forward(_: *mut cef_browser_t) {
    core::send_window_event(NavigationWindowEvent(Forward));
}

extern "C" fn get_main_frame(browser: *mut cef_browser_t) -> *mut cef_frame_t {
    unsafe {
        let result = (*browser).extra.frame;
        eutil::add_ref(&mut (*result).base);
        result
    }
}

extern "C" fn get_host(browser: *mut cef_browser_t) -> *mut cef_browser_host_t {
    unsafe {
        let result = (*browser).extra.host;
        eutil::add_ref(&mut (*result).base);
        result
    }
}

