/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use browser::CefBrowser;
use types::{PET_VIEW, cef_rect_t, cef_render_handler_t};

use libc::c_void;
use std::mem;
use std::ptr;

def_cef_object!(CefRenderHandler, cef_render_handler_t)

impl CefRenderHandler {
    pub fn get_view_rect(&self, browser: &CefBrowser) -> cef_rect_t {
        unsafe {
            let mut rect = mem::uninitialized();
            ((*self.c_object).get_view_rect)(self.c_object,
                                             browser.c_object_addrefed(),
                                             &mut rect);
            rect
        }
    }

    pub fn paint(&self, browser: CefBrowser) {
        unsafe {
            ((*self.c_object).on_paint)(self.c_object,
                                        browser.c_object_addrefed(),
                                        PET_VIEW,
                                        0,
                                        ptr::null(),
                                        (&0 as *const u8 as *const c_void),
                                        0,
                                        0)
        }
    }

    pub fn present(&self, browser: CefBrowser) {
        unsafe {
            ((*self.c_object).on_present)(self.c_object, browser.c_object_addrefed())
        }
    }

    pub fn get_backing_rect(&self, browser: &CefBrowser) -> cef_rect_t {
        unsafe {
            let mut rect = mem::uninitialized();
            ((*self.c_object).get_backing_rect)(self.c_object,
                                                browser.c_object_addrefed(),
                                                &mut rect);
            rect
        }
    }
}

