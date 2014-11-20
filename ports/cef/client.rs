/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use render_handler::CefRenderHandler;
use types::cef_client_t;

def_cef_object!(CefClient, cef_client_t)

impl CefClient {
    pub fn get_render_handler(&self) -> CefRenderHandler {
        unsafe {
            CefRenderHandler::from_c_object_addref(((*self.c_object).get_render_handler)(
                    self.c_object))
        }
    }
}

