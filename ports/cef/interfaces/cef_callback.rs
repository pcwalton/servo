// Copyright (c) 2014 Marshall A. Greenblatt. All rights reserved.
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions are
// met:
//
//    * Redistributions of source code must retain the above copyright
// notice, this list of conditions and the following disclaimer.
//    * Redistributions in binary form must reproduce the above
// copyright notice, this list of conditions and the following disclaimer
// in the documentation and/or other materials provided with the
// distribution.
//    * Neither the name of Google Inc. nor the name Chromium Embedded
// Framework nor the names of its contributors may be used to endorse
// or promote products derived from this software without specific prior
// written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
// "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
// LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
// A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
// OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
// SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
// LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
// DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
// THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
// (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
//
// ---------------------------------------------------------------------------
//
// This file was generated by the CEF translator tool and should not be edited
// by hand. See the translator.README.txt file in the tools directory for
// more information.
//

#![allow(non_snake_case, unused_imports)]

use eutil;
use interfaces;
use types;
use wrappers::CefWrap;

use libc;
use std::collections::HashMap;
use std::ptr;

//
// Generic callback structure used for asynchronous continuation.
//
#[repr(C)]
pub struct _cef_callback_t {
  //
  // Base structure.
  //
  pub base: types::cef_base_t,

  //
  // Continue processing.
  //
  pub cont: Option<extern "C" fn(this: *mut cef_callback_t) -> ()>,

  //
  // Cancel processing.
  //
  pub cancel: Option<extern "C" fn(this: *mut cef_callback_t) -> ()>,

  //
  // The reference count. This will only be present for Rust instances!
  //
  ref_count: uint,

  //
  // Extra data. This will only be present for Rust instances!
  //
  pub extra: u8,
} 

pub type cef_callback_t = _cef_callback_t;


//
// Generic callback structure used for asynchronous continuation.
//
pub struct CefCallback {
  c_object: *mut cef_callback_t,
}

impl Clone for CefCallback {
  fn clone(&self) -> CefCallback{
    unsafe {
      if !self.c_object.is_null() {
        ((*self.c_object).base.add_ref.unwrap())(&mut (*self.c_object).base);
      }
      CefCallback {
        c_object: self.c_object,
      }
    }
  }
}

impl Drop for CefCallback {
  fn drop(&mut self) {
    unsafe {
      if !self.c_object.is_null() {
        ((*self.c_object).base.release.unwrap())(&mut (*self.c_object).base);
      }
    }
  }
}

impl CefCallback {
  pub unsafe fn from_c_object(c_object: *mut cef_callback_t) -> CefCallback {
    CefCallback {
      c_object: c_object,
    }
  }

  pub unsafe fn from_c_object_addref(c_object: *mut cef_callback_t) -> CefCallback {
    if !c_object.is_null() {
      ((*c_object).base.add_ref.unwrap())(&mut (*c_object).base);
    }
    CefCallback {
      c_object: c_object,
    }
  }

  pub fn c_object(&self) -> *mut cef_callback_t {
    self.c_object
  }

  pub fn c_object_addrefed(&self) -> *mut cef_callback_t {
    unsafe {
      if !self.c_object.is_null() {
        eutil::add_ref(self.c_object as *mut types::cef_base_t);
      }
      self.c_object
    }
  }

  pub fn is_null_cef_object(&self) -> bool {
    self.c_object.is_null()
  }
  pub fn is_not_null_cef_object(&self) -> bool {
    !self.c_object.is_null()
  }

  //
  // Continue processing.
  //
  pub fn cont(&self) -> () {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).cont.unwrap())(
          self.c_object))
    }
  }

  //
  // Cancel processing.
  //
  pub fn cancel(&self) -> () {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).cancel.unwrap())(
          self.c_object))
    }
  }
} 

impl CefWrap<*mut cef_callback_t> for CefCallback {
  fn to_c(rust_object: CefCallback) -> *mut cef_callback_t {
    rust_object.c_object_addrefed()
  }
  unsafe fn to_rust(c_object: *mut cef_callback_t) -> CefCallback {
    CefCallback::from_c_object_addref(c_object)
  }
}
impl CefWrap<*mut cef_callback_t> for Option<CefCallback> {
  fn to_c(rust_object: Option<CefCallback>) -> *mut cef_callback_t {
    match rust_object {
      None => ptr::null_mut(),
      Some(rust_object) => rust_object.c_object_addrefed(),
    }
  }
  unsafe fn to_rust(c_object: *mut cef_callback_t) -> Option<CefCallback> {
    if c_object.is_null() {
      None
    } else {
      Some(CefCallback::from_c_object_addref(c_object))
    }
  }
}


//
// Generic callback structure used for asynchronous completion.
//
#[repr(C)]
pub struct _cef_completion_callback_t {
  //
  // Base structure.
  //
  pub base: types::cef_base_t,

  //
  // Method that will be called once the task is complete.
  //
  pub on_complete: Option<extern "C" fn(
      this: *mut cef_completion_callback_t) -> ()>,

  //
  // The reference count. This will only be present for Rust instances!
  //
  ref_count: uint,

  //
  // Extra data. This will only be present for Rust instances!
  //
  pub extra: u8,
} 

pub type cef_completion_callback_t = _cef_completion_callback_t;


//
// Generic callback structure used for asynchronous completion.
//
pub struct CefCompletionCallback {
  c_object: *mut cef_completion_callback_t,
}

impl Clone for CefCompletionCallback {
  fn clone(&self) -> CefCompletionCallback{
    unsafe {
      if !self.c_object.is_null() {
        ((*self.c_object).base.add_ref.unwrap())(&mut (*self.c_object).base);
      }
      CefCompletionCallback {
        c_object: self.c_object,
      }
    }
  }
}

impl Drop for CefCompletionCallback {
  fn drop(&mut self) {
    unsafe {
      if !self.c_object.is_null() {
        ((*self.c_object).base.release.unwrap())(&mut (*self.c_object).base);
      }
    }
  }
}

impl CefCompletionCallback {
  pub unsafe fn from_c_object(c_object: *mut cef_completion_callback_t) -> CefCompletionCallback {
    CefCompletionCallback {
      c_object: c_object,
    }
  }

  pub unsafe fn from_c_object_addref(c_object: *mut cef_completion_callback_t) -> CefCompletionCallback {
    if !c_object.is_null() {
      ((*c_object).base.add_ref.unwrap())(&mut (*c_object).base);
    }
    CefCompletionCallback {
      c_object: c_object,
    }
  }

  pub fn c_object(&self) -> *mut cef_completion_callback_t {
    self.c_object
  }

  pub fn c_object_addrefed(&self) -> *mut cef_completion_callback_t {
    unsafe {
      if !self.c_object.is_null() {
        eutil::add_ref(self.c_object as *mut types::cef_base_t);
      }
      self.c_object
    }
  }

  pub fn is_null_cef_object(&self) -> bool {
    self.c_object.is_null()
  }
  pub fn is_not_null_cef_object(&self) -> bool {
    !self.c_object.is_null()
  }

  //
  // Method that will be called once the task is complete.
  //
  pub fn on_complete(&self) -> () {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).on_complete.unwrap())(
          self.c_object))
    }
  }
} 

impl CefWrap<*mut cef_completion_callback_t> for CefCompletionCallback {
  fn to_c(rust_object: CefCompletionCallback) -> *mut cef_completion_callback_t {
    rust_object.c_object_addrefed()
  }
  unsafe fn to_rust(c_object: *mut cef_completion_callback_t) -> CefCompletionCallback {
    CefCompletionCallback::from_c_object_addref(c_object)
  }
}
impl CefWrap<*mut cef_completion_callback_t> for Option<CefCompletionCallback> {
  fn to_c(rust_object: Option<CefCompletionCallback>) -> *mut cef_completion_callback_t {
    match rust_object {
      None => ptr::null_mut(),
      Some(rust_object) => rust_object.c_object_addrefed(),
    }
  }
  unsafe fn to_rust(c_object: *mut cef_completion_callback_t) -> Option<CefCompletionCallback> {
    if c_object.is_null() {
      None
    } else {
      Some(CefCompletionCallback::from_c_object_addref(c_object))
    }
  }
}

