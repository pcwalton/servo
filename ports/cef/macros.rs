/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![macro_escape]

macro_rules! def_cef_object(
    ($rust_name:ident, $c_name:ident) => (
        pub struct $rust_name {
            c_object: *mut $c_name,
        }

        impl Clone for $rust_name {
            fn clone(&self) -> $rust_name {
                unsafe {
                    ((*self.c_object).base.add_ref.unwrap())(&mut (*self.c_object).base);
                    $rust_name {
                        c_object: self.c_object,
                    }
                }
            }
        }

        impl Drop for $rust_name {
            fn drop(&mut self) {
                unsafe {
                    ((*self.c_object).base.release.unwrap())(&mut (*self.c_object).base);
                }
            }
        }

        impl $rust_name {
            pub fn from_c_object(c_object: *mut $c_name) -> $rust_name {
                $rust_name {
                    c_object: c_object,
                }
            }

            pub fn from_c_object_addref(c_object: *mut $c_name) -> $rust_name {
                unsafe {
                    ((*c_object).base.add_ref.unwrap())(&mut (*c_object).base);
                }
                $rust_name {
                    c_object: c_object,
                }
            }

            pub fn c_object(&self) -> *mut $c_name {
                self.c_object
            }

            pub fn c_object_addrefed(&self) -> *mut $c_name {
                unsafe {
                    ::eutil::add_ref(self.c_object as *mut ::types::cef_base_t);
                }
                self.c_object
            }
        }
    )
)

