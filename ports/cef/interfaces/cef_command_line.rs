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
// Structure used to create and/or parse command line arguments. Arguments with
// '--', '-' and, on Windows, '/' prefixes are considered switches. Switches
// will always precede any arguments without switch prefixes. Switches can
// optionally have a value specified using the '=' delimiter (e.g.
// "-switch=value"). An argument of "--" will terminate switch parsing with all
// subsequent tokens, regardless of prefix, being interpreted as non-switch
// arguments. Switch names are considered case-insensitive. This structure can
// be used before cef_initialize() is called.
//
#[repr(C)]
pub struct _cef_command_line_t {
  //
  // Base structure.
  //
  pub base: types::cef_base_t,

  //
  // Returns true (1) if this object is valid. Do not call any other functions
  // if this function returns false (0).
  //
  pub is_valid: Option<extern "C" fn(
      this: *mut cef_command_line_t) -> libc::c_int>,

  //
  // Returns true (1) if the values of this object are read-only. Some APIs may
  // expose read-only objects.
  //
  pub is_read_only: Option<extern "C" fn(
      this: *mut cef_command_line_t) -> libc::c_int>,

  //
  // Returns a writable copy of this object.
  //
  pub copy: Option<extern "C" fn(
      this: *mut cef_command_line_t) -> *mut interfaces::cef_command_line_t>,

  //
  // Initialize the command line with the specified |argc| and |argv| values.
  // The first argument must be the name of the program. This function is only
  // supported on non-Windows platforms.
  //
  pub init_from_argv: Option<extern "C" fn(this: *mut cef_command_line_t,
      argc: libc::c_int, argv: *const *const libc::c_char) -> ()>,

  //
  // Initialize the command line with the string returned by calling
  // GetCommandLineW(). This function is only supported on Windows.
  //
  pub init_from_string: Option<extern "C" fn(this: *mut cef_command_line_t,
      command_line: *const types::cef_string_t) -> ()>,

  //
  // Reset the command-line switches and arguments but leave the program
  // component unchanged.
  //
  pub reset: Option<extern "C" fn(this: *mut cef_command_line_t) -> ()>,

  //
  // Retrieve the original command line string as a vector of strings. The argv
  // array: { program, [(--|-|/)switch[=value]]*, [--], [argument]* }
  //
  pub get_argv: Option<extern "C" fn(this: *mut cef_command_line_t,
      argv: types::cef_string_list_t) -> ()>,

  //
  // Constructs and returns the represented command line string. Use this
  // function cautiously because quoting behavior is unclear.
  //
  // The resulting string must be freed by calling cef_string_userfree_free().
  pub get_command_line_string: Option<extern "C" fn(
      this: *mut cef_command_line_t) -> types::cef_string_userfree_t>,

  //
  // Get the program part of the command line string (the first item).
  //
  // The resulting string must be freed by calling cef_string_userfree_free().
  pub get_program: Option<extern "C" fn(
      this: *mut cef_command_line_t) -> types::cef_string_userfree_t>,

  //
  // Set the program part of the command line string (the first item).
  //
  pub set_program: Option<extern "C" fn(this: *mut cef_command_line_t,
      program: *const types::cef_string_t) -> ()>,

  //
  // Returns true (1) if the command line has switches.
  //
  pub has_switches: Option<extern "C" fn(
      this: *mut cef_command_line_t) -> libc::c_int>,

  //
  // Returns true (1) if the command line contains the given switch.
  //
  pub has_switch: Option<extern "C" fn(this: *mut cef_command_line_t,
      name: *const types::cef_string_t) -> libc::c_int>,

  //
  // Returns the value associated with the given switch. If the switch has no
  // value or isn't present this function returns the NULL string.
  //
  // The resulting string must be freed by calling cef_string_userfree_free().
  pub get_switch_value: Option<extern "C" fn(this: *mut cef_command_line_t,
      name: *const types::cef_string_t) -> types::cef_string_userfree_t>,

  //
  // Returns the map of switch names and values. If a switch has no value an
  // NULL string is returned.
  //
  pub get_switches: Option<extern "C" fn(this: *mut cef_command_line_t,
      switches: types::cef_string_map_t) -> ()>,

  //
  // Add a switch to the end of the command line. If the switch has no value
  // pass an NULL value string.
  //
  pub append_switch: Option<extern "C" fn(this: *mut cef_command_line_t,
      name: *const types::cef_string_t) -> ()>,

  //
  // Add a switch with the specified value to the end of the command line.
  //
  pub append_switch_with_value: Option<extern "C" fn(
      this: *mut cef_command_line_t, name: *const types::cef_string_t,
      value: *const types::cef_string_t) -> ()>,

  //
  // True if there are remaining command line arguments.
  //
  pub has_arguments: Option<extern "C" fn(
      this: *mut cef_command_line_t) -> libc::c_int>,

  //
  // Get the remaining command line arguments.
  //
  pub get_arguments: Option<extern "C" fn(this: *mut cef_command_line_t,
      arguments: types::cef_string_list_t) -> ()>,

  //
  // Add an argument to the end of the command line.
  //
  pub append_argument: Option<extern "C" fn(this: *mut cef_command_line_t,
      argument: *const types::cef_string_t) -> ()>,

  //
  // Insert a command before the current command. Common for debuggers, like
  // "valgrind" or "gdb --args".
  //
  pub prepend_wrapper: Option<extern "C" fn(this: *mut cef_command_line_t,
      wrapper: *const types::cef_string_t) -> ()>,

  //
  // The reference count. This will only be present for Rust instances!
  //
  pub ref_count: uint,

  //
  // Extra data. This will only be present for Rust instances!
  //
  pub extra: u8,
} 

pub type cef_command_line_t = _cef_command_line_t;


//
// Structure used to create and/or parse command line arguments. Arguments with
// '--', '-' and, on Windows, '/' prefixes are considered switches. Switches
// will always precede any arguments without switch prefixes. Switches can
// optionally have a value specified using the '=' delimiter (e.g.
// "-switch=value"). An argument of "--" will terminate switch parsing with all
// subsequent tokens, regardless of prefix, being interpreted as non-switch
// arguments. Switch names are considered case-insensitive. This structure can
// be used before cef_initialize() is called.
//
pub struct CefCommandLine {
  c_object: *mut cef_command_line_t,
}

impl Clone for CefCommandLine {
  fn clone(&self) -> CefCommandLine{
    unsafe {
      if !self.c_object.is_null() {
        ((*self.c_object).base.add_ref.unwrap())(&mut (*self.c_object).base);
      }
      CefCommandLine {
        c_object: self.c_object,
      }
    }
  }
}

impl Drop for CefCommandLine {
  fn drop(&mut self) {
    unsafe {
      if !self.c_object.is_null() {
        ((*self.c_object).base.release.unwrap())(&mut (*self.c_object).base);
      }
    }
  }
}

impl CefCommandLine {
  pub unsafe fn from_c_object(c_object: *mut cef_command_line_t) -> CefCommandLine {
    CefCommandLine {
      c_object: c_object,
    }
  }

  pub unsafe fn from_c_object_addref(c_object: *mut cef_command_line_t) -> CefCommandLine {
    if !c_object.is_null() {
      ((*c_object).base.add_ref.unwrap())(&mut (*c_object).base);
    }
    CefCommandLine {
      c_object: c_object,
    }
  }

  pub fn c_object(&self) -> *mut cef_command_line_t {
    self.c_object
  }

  pub fn c_object_addrefed(&self) -> *mut cef_command_line_t {
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
  pub fn copy(&self) -> interfaces::CefCommandLine {
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
  // Initialize the command line with the specified |argc| and |argv| values.
  // The first argument must be the name of the program. This function is only
  // supported on non-Windows platforms.
  //
  pub fn init_from_argv(&self, argc: libc::c_int, argv: &&str) -> () {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).init_from_argv.unwrap())(
          self.c_object,
          CefWrap::to_c(argc),
          CefWrap::to_c(argv)))
    }
  }

  //
  // Initialize the command line with the string returned by calling
  // GetCommandLineW(). This function is only supported on Windows.
  //
  pub fn init_from_string(&self, command_line: &[u16]) -> () {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).init_from_string.unwrap())(
          self.c_object,
          CefWrap::to_c(command_line)))
    }
  }

  //
  // Reset the command-line switches and arguments but leave the program
  // component unchanged.
  //
  pub fn reset(&self) -> () {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).reset.unwrap())(
          self.c_object))
    }
  }

  //
  // Retrieve the original command line string as a vector of strings. The argv
  // array: { program, [(--|-|/)switch[=value]]*, [--], [argument]* }
  //
  pub fn get_argv(&self, argv: Vec<String>) -> () {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).get_argv.unwrap())(
          self.c_object,
          CefWrap::to_c(argv)))
    }
  }

  //
  // Constructs and returns the represented command line string. Use this
  // function cautiously because quoting behavior is unclear.
  //
  // The resulting string must be freed by calling cef_string_userfree_free().
  pub fn get_command_line_string(&self) -> String {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).get_command_line_string.unwrap())(
          self.c_object))
    }
  }

  //
  // Get the program part of the command line string (the first item).
  //
  // The resulting string must be freed by calling cef_string_userfree_free().
  pub fn get_program(&self) -> String {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).get_program.unwrap())(
          self.c_object))
    }
  }

  //
  // Set the program part of the command line string (the first item).
  //
  pub fn set_program(&self, program: &[u16]) -> () {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).set_program.unwrap())(
          self.c_object,
          CefWrap::to_c(program)))
    }
  }

  //
  // Returns true (1) if the command line has switches.
  //
  pub fn has_switches(&self) -> libc::c_int {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).has_switches.unwrap())(
          self.c_object))
    }
  }

  //
  // Returns true (1) if the command line contains the given switch.
  //
  pub fn has_switch(&self, name: &[u16]) -> libc::c_int {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).has_switch.unwrap())(
          self.c_object,
          CefWrap::to_c(name)))
    }
  }

  //
  // Returns the value associated with the given switch. If the switch has no
  // value or isn't present this function returns the NULL string.
  //
  // The resulting string must be freed by calling cef_string_userfree_free().
  pub fn get_switch_value(&self, name: &[u16]) -> String {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).get_switch_value.unwrap())(
          self.c_object,
          CefWrap::to_c(name)))
    }
  }

  //
  // Returns the map of switch names and values. If a switch has no value an
  // NULL string is returned.
  //
  pub fn get_switches(&self, switches: HashMap<String,String>) -> () {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).get_switches.unwrap())(
          self.c_object,
          CefWrap::to_c(switches)))
    }
  }

  //
  // Add a switch to the end of the command line. If the switch has no value
  // pass an NULL value string.
  //
  pub fn append_switch(&self, name: &[u16]) -> () {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).append_switch.unwrap())(
          self.c_object,
          CefWrap::to_c(name)))
    }
  }

  //
  // Add a switch with the specified value to the end of the command line.
  //
  pub fn append_switch_with_value(&self, name: &[u16], value: &[u16]) -> () {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).append_switch_with_value.unwrap())(
          self.c_object,
          CefWrap::to_c(name),
          CefWrap::to_c(value)))
    }
  }

  //
  // True if there are remaining command line arguments.
  //
  pub fn has_arguments(&self) -> libc::c_int {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).has_arguments.unwrap())(
          self.c_object))
    }
  }

  //
  // Get the remaining command line arguments.
  //
  pub fn get_arguments(&self, arguments: Vec<String>) -> () {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).get_arguments.unwrap())(
          self.c_object,
          CefWrap::to_c(arguments)))
    }
  }

  //
  // Add an argument to the end of the command line.
  //
  pub fn append_argument(&self, argument: &[u16]) -> () {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).append_argument.unwrap())(
          self.c_object,
          CefWrap::to_c(argument)))
    }
  }

  //
  // Insert a command before the current command. Common for debuggers, like
  // "valgrind" or "gdb --args".
  //
  pub fn prepend_wrapper(&self, wrapper: &[u16]) -> () {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).prepend_wrapper.unwrap())(
          self.c_object,
          CefWrap::to_c(wrapper)))
    }
  }

  //
  // Create a new cef_command_line_t instance.
  //
  pub fn create_command_line() -> interfaces::CefCommandLine {
    unsafe {
      CefWrap::to_rust(
        ::command_line::cef_command_line_create_command_line(
))
    }
  }

  //
  // Returns the singleton global cef_command_line_t object. The returned object
  // will be read-only.
  //
  pub fn get_global_command_line() -> interfaces::CefCommandLine {
    unsafe {
      CefWrap::to_rust(
        ::command_line::cef_command_line_get_global_command_line(
))
    }
  }
} 

impl CefWrap<*mut cef_command_line_t> for CefCommandLine {
  fn to_c(rust_object: CefCommandLine) -> *mut cef_command_line_t {
    rust_object.c_object_addrefed()
  }
  unsafe fn to_rust(c_object: *mut cef_command_line_t) -> CefCommandLine {
    CefCommandLine::from_c_object_addref(c_object)
  }
}
impl CefWrap<*mut cef_command_line_t> for Option<CefCommandLine> {
  fn to_c(rust_object: Option<CefCommandLine>) -> *mut cef_command_line_t {
    match rust_object {
      None => ptr::null_mut(),
      Some(rust_object) => rust_object.c_object_addrefed(),
    }
  }
  unsafe fn to_rust(c_object: *mut cef_command_line_t) -> Option<CefCommandLine> {
    if c_object.is_null() {
      None
    } else {
      Some(CefCommandLine::from_c_object_addref(c_object))
    }
  }
}

