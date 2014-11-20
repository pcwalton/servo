/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use types::cef_base_t;

use libc::{mod, c_int, c_void, size_t};
use std::mem;
use std::slice;
use std::str;

pub fn slice_to_str(s: *const u8, l: uint, f: |&str| -> c_int) -> c_int {
    unsafe {
        slice::raw::buf_as_slice(s, l, |result| {
             str::from_utf8(result).map(|s| f(s)).unwrap_or(0)
        })
    }
}

/// Creates a new raw CEF object of the given type and sets up its reference counting machinery.
/// All fields are initialized to zero. It is the caller's responsibility to ensure that the given
/// type is a CEF type with `cef_base_t` as its first member.
///
/// `ExtraType` is the associated Servo "extra" type. It must have the object's reference count as
/// an unsigned pointer-sized integer as its first member.
pub unsafe fn create_cef_object<Base,Extra>() -> *mut Base {
    let object = libc::calloc(1, mem::size_of::<Base>() as size_t) as *mut cef_base_t;
    (*object).size = (mem::size_of::<Base>() as size_t) - (mem::size_of::<Extra>() as size_t);
    (*object).add_ref = Some(servo_add_ref);
    (*object).release = Some(servo_release);
    *ref_count(object) = 1;
    object as *mut Base
}

/// Returns a pointer to the Servo-specific reference count for the given object. This only works
/// on objects that Servo created!
unsafe fn ref_count(object: *mut cef_base_t) -> *mut uint {
    // The reference count should be the first field of the extra data.
    (object as *mut u8).offset((*object).size as int) as *mut uint
}

/// Increments the reference count on a CEF object. This only works on objects that Servo created!
extern "C" fn servo_add_ref(object: *mut cef_base_t) -> c_int {
    unsafe {
        let count = ref_count(object);
        *count += 1;
        *count as c_int
    }
}

/// Decrements the reference count on a CEF object. If zero, frees it. This only works on objects
/// that Servo created!
extern "C" fn servo_release(object: *mut cef_base_t) -> c_int {
    unsafe {
        let count = ref_count(object);
        *count -= 1;
        let new_count = *count;
        if new_count == 0 {
            servo_free(object);
        }
        new_count as c_int
    }
}

unsafe fn servo_free(object: *mut cef_base_t) {
    println!("freeing Servo-created CEF object!");
    libc::free(object as *mut c_void);
}

pub unsafe fn add_ref(c_object: *mut cef_base_t) {
    ((*c_object).add_ref.unwrap())(c_object);
}

