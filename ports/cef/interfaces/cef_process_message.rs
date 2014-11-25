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
// Structure representing a message. Can be used on any process and thread.
//
#[repr(C)]
pub struct _cef_process_message_t {
  //
  // Base structure.
  //
  pub base: types::cef_base_t,

  //
  // Returns true (1) if this object is valid. Do not call any other functions
  // if this function returns false (0).
  //
  pub is_valid: Option<extern "C" fn(
      this: *mut cef_process_message_t) -> libc::c_int>,

  //
  // Returns true (1) if the values of this object are read-only. Some APIs may
  // expose read-only objects.
  //
  pub is_read_only: Option<extern "C" fn(
      this: *mut cef_process_message_t) -> libc::c_int>,

  //
  // Returns a writable copy of this object.
  //
  pub copy: Option<extern "C" fn(
      this: *mut cef_process_message_t) -> *mut interfaces::cef_process_message_t>,

  //
  // Returns the message name.
  //
  // The resulting string must be freed by calling cef_string_userfree_free().
  pub get_name: Option<extern "C" fn(
      this: *mut cef_process_message_t) -> types::cef_string_userfree_t>,

  //
  // Returns the list of arguments.
  //
  pub get_argument_list: Option<extern "C" fn(
      this: *mut cef_process_message_t) -> *mut interfaces::cef_list_value_t>,

  //
  // The reference count. This will only be present for Rust instances!
  //
  ref_count: uint,

  //
  // Extra data. This will only be present for Rust instances!
  //
  pub extra: u8,
} 

pub type cef_process_message_t = _cef_process_message_t;


//
// Structure representing a message. Can be used on any process and thread.
//
pub struct CefProcessMessage {
  c_object: *mut cef_process_message_t,
}

impl Clone for CefProcessMessage {
  fn clone(&self) -> CefProcessMessage{
    unsafe {
      if !self.c_object.is_null() {
        ((*self.c_object).base.add_ref.unwrap())(&mut (*self.c_object).base);
      }
      CefProcessMessage {
        c_object: self.c_object,
      }
    }
  }
}

impl Drop for CefProcessMessage {
  fn drop(&mut self) {
    unsafe {
      if !self.c_object.is_null() {
        ((*self.c_object).base.release.unwrap())(&mut (*self.c_object).base);
      }
    }
  }
}

impl CefProcessMessage {
  pub unsafe fn from_c_object(c_object: *mut cef_process_message_t) -> CefProcessMessage {
    CefProcessMessage {
      c_object: c_object,
    }
  }

  pub unsafe fn from_c_object_addref(c_object: *mut cef_process_message_t) -> CefProcessMessage {
    if !c_object.is_null() {
      ((*c_object).base.add_ref.unwrap())(&mut (*c_object).base);
    }
    CefProcessMessage {
      c_object: c_object,
    }
  }

  pub fn c_object(&self) -> *mut cef_process_message_t {
    self.c_object
  }

  pub fn c_object_addrefed(&self) -> *mut cef_process_message_t {
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
  // Returns true (1) if this object is valid. Do not call any other functions
  // if this function returns false (0).
  //
  pub fn is_valid(&self) -> libc::c_int {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).is_valid.unwrap())(
          self.c_object))
    }
  }

  //
  // Returns true (1) if the values of this object are read-only. Some APIs may
  // expose read-only objects.
  //
  pub fn is_read_only(&self) -> libc::c_int {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).is_read_only.unwrap())(
          self.c_object))
    }
  }

  //
  // Returns a writable copy of this object.
  //
  pub fn copy(&self) -> interfaces::CefProcessMessage {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).copy.unwrap())(
          self.c_object))
    }
  }

  //
  // Returns the message name.
  //
  // The resulting string must be freed by calling cef_string_userfree_free().
  pub fn get_name(&self) -> String {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).get_name.unwrap())(
          self.c_object))
    }
  }

  //
  // Returns the list of arguments.
  //
  pub fn get_argument_list(&self) -> interfaces::CefListValue {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).get_argument_list.unwrap())(
          self.c_object))
    }
  }

  //
  // Create a new cef_process_message_t object with the specified name.
  //
  pub fn create(name: &str) -> interfaces::CefProcessMessage {
    unsafe {
      CefWrap::to_rust(
        ::process_message::cef_process_message_create(
          CefWrap::to_c(name)))
    }
  }
} 

impl CefWrap<*mut cef_process_message_t> for CefProcessMessage {
  fn to_c(rust_object: CefProcessMessage) -> *mut cef_process_message_t {
    rust_object.c_object_addrefed()
  }
  unsafe fn to_rust(c_object: *mut cef_process_message_t) -> CefProcessMessage {
    CefProcessMessage::from_c_object_addref(c_object)
  }
}
impl CefWrap<*mut cef_process_message_t> for Option<CefProcessMessage> {
  fn to_c(rust_object: Option<CefProcessMessage>) -> *mut cef_process_message_t {
    match rust_object {
      None => ptr::null_mut(),
      Some(rust_object) => rust_object.c_object_addrefed(),
    }
  }
  unsafe fn to_rust(c_object: *mut cef_process_message_t) -> Option<CefProcessMessage> {
    if c_object.is_null() {
      None
    } else {
      Some(CefProcessMessage::from_c_object_addref(c_object))
    }
  }
}

