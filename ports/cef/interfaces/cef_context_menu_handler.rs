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
// Implement this structure to handle context menu events. The functions of this
// structure will be called on the UI thread.
//
#[repr(C)]
pub struct _cef_context_menu_handler_t {
  //
  // Base structure.
  //
  pub base: types::cef_base_t,

  //
  // Called before a context menu is displayed. |params| provides information
  // about the context menu state. |model| initially contains the default
  // context menu. The |model| can be cleared to show no context menu or
  // modified to show a custom menu. Do not keep references to |params| or
  // |model| outside of this callback.
  //
  pub on_before_context_menu: Option<extern "C" fn(
      this: *mut cef_context_menu_handler_t,
      browser: *mut interfaces::cef_browser_t,
      frame: *mut interfaces::cef_frame_t,
      params: *mut interfaces::cef_context_menu_params_t,
      model: *mut interfaces::cef_menu_model_t) -> ()>,

  //
  // Called to execute a command selected from the context menu. Return true (1)
  // if the command was handled or false (0) for the default implementation. See
  // cef_menu_id_t for the command ids that have default implementations. All
  // user-defined command ids should be between MENU_ID_USER_FIRST and
  // MENU_ID_USER_LAST. |params| will have the same values as what was passed to
  // on_before_context_menu(). Do not keep a reference to |params| outside of
  // this callback.
  //
  pub on_context_menu_command: Option<extern "C" fn(
      this: *mut cef_context_menu_handler_t,
      browser: *mut interfaces::cef_browser_t,
      frame: *mut interfaces::cef_frame_t,
      params: *mut interfaces::cef_context_menu_params_t,
      command_id: libc::c_int,
      event_flags: types::cef_event_flags_t) -> libc::c_int>,

  //
  // Called when the context menu is dismissed irregardless of whether the menu
  // was NULL or a command was selected.
  //
  pub on_context_menu_dismissed: Option<extern "C" fn(
      this: *mut cef_context_menu_handler_t,
      browser: *mut interfaces::cef_browser_t,
      frame: *mut interfaces::cef_frame_t) -> ()>,

  //
  // The reference count. This will only be present for Rust instances!
  //
  ref_count: uint,

  //
  // Extra data. This will only be present for Rust instances!
  //
  pub extra: u8,
} 

pub type cef_context_menu_handler_t = _cef_context_menu_handler_t;


//
// Implement this structure to handle context menu events. The functions of this
// structure will be called on the UI thread.
//
pub struct CefContextMenuHandler {
  c_object: *mut cef_context_menu_handler_t,
}

impl Clone for CefContextMenuHandler {
  fn clone(&self) -> CefContextMenuHandler{
    unsafe {
      if !self.c_object.is_null() {
        ((*self.c_object).base.add_ref.unwrap())(&mut (*self.c_object).base);
      }
      CefContextMenuHandler {
        c_object: self.c_object,
      }
    }
  }
}

impl Drop for CefContextMenuHandler {
  fn drop(&mut self) {
    unsafe {
      if !self.c_object.is_null() {
        ((*self.c_object).base.release.unwrap())(&mut (*self.c_object).base);
      }
    }
  }
}

impl CefContextMenuHandler {
  pub unsafe fn from_c_object(c_object: *mut cef_context_menu_handler_t) -> CefContextMenuHandler {
    CefContextMenuHandler {
      c_object: c_object,
    }
  }

  pub unsafe fn from_c_object_addref(c_object: *mut cef_context_menu_handler_t) -> CefContextMenuHandler {
    if !c_object.is_null() {
      ((*c_object).base.add_ref.unwrap())(&mut (*c_object).base);
    }
    CefContextMenuHandler {
      c_object: c_object,
    }
  }

  pub fn c_object(&self) -> *mut cef_context_menu_handler_t {
    self.c_object
  }

  pub fn c_object_addrefed(&self) -> *mut cef_context_menu_handler_t {
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
  // Called before a context menu is displayed. |params| provides information
  // about the context menu state. |model| initially contains the default
  // context menu. The |model| can be cleared to show no context menu or
  // modified to show a custom menu. Do not keep references to |params| or
  // |model| outside of this callback.
  //
  pub fn on_before_context_menu(&self, browser: interfaces::CefBrowser,
      frame: interfaces::CefFrame, params: interfaces::CefContextMenuParams,
      model: interfaces::CefMenuModel) -> () {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).on_before_context_menu.unwrap())(
          self.c_object,
          CefWrap::to_c(browser),
          CefWrap::to_c(frame),
          CefWrap::to_c(params),
          CefWrap::to_c(model)))
    }
  }

  //
  // Called to execute a command selected from the context menu. Return true (1)
  // if the command was handled or false (0) for the default implementation. See
  // cef_menu_id_t for the command ids that have default implementations. All
  // user-defined command ids should be between MENU_ID_USER_FIRST and
  // MENU_ID_USER_LAST. |params| will have the same values as what was passed to
  // on_before_context_menu(). Do not keep a reference to |params| outside of
  // this callback.
  //
  pub fn on_context_menu_command(&self, browser: interfaces::CefBrowser,
      frame: interfaces::CefFrame, params: interfaces::CefContextMenuParams,
      command_id: libc::c_int,
      event_flags: types::cef_event_flags_t) -> libc::c_int {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).on_context_menu_command.unwrap())(
          self.c_object,
          CefWrap::to_c(browser),
          CefWrap::to_c(frame),
          CefWrap::to_c(params),
          CefWrap::to_c(command_id),
          CefWrap::to_c(event_flags)))
    }
  }

  //
  // Called when the context menu is dismissed irregardless of whether the menu
  // was NULL or a command was selected.
  //
  pub fn on_context_menu_dismissed(&self, browser: interfaces::CefBrowser,
      frame: interfaces::CefFrame) -> () {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).on_context_menu_dismissed.unwrap())(
          self.c_object,
          CefWrap::to_c(browser),
          CefWrap::to_c(frame)))
    }
  }
} 

impl CefWrap<*mut cef_context_menu_handler_t> for CefContextMenuHandler {
  fn to_c(rust_object: CefContextMenuHandler) -> *mut cef_context_menu_handler_t {
    rust_object.c_object_addrefed()
  }
  unsafe fn to_rust(c_object: *mut cef_context_menu_handler_t) -> CefContextMenuHandler {
    CefContextMenuHandler::from_c_object_addref(c_object)
  }
}
impl CefWrap<*mut cef_context_menu_handler_t> for Option<CefContextMenuHandler> {
  fn to_c(rust_object: Option<CefContextMenuHandler>) -> *mut cef_context_menu_handler_t {
    match rust_object {
      None => ptr::null_mut(),
      Some(rust_object) => rust_object.c_object_addrefed(),
    }
  }
  unsafe fn to_rust(c_object: *mut cef_context_menu_handler_t) -> Option<CefContextMenuHandler> {
    if c_object.is_null() {
      None
    } else {
      Some(CefContextMenuHandler::from_c_object_addref(c_object))
    }
  }
}


//
// Provides information about the context menu state. The ethods of this
// structure can only be accessed on browser process the UI thread.
//
#[repr(C)]
pub struct _cef_context_menu_params_t {
  //
  // Base structure.
  //
  pub base: types::cef_base_t,

  //
  // Returns the X coordinate of the mouse where the context menu was invoked.
  // Coords are relative to the associated RenderView's origin.
  //
  pub get_xcoord: Option<extern "C" fn(
      this: *mut cef_context_menu_params_t) -> libc::c_int>,

  //
  // Returns the Y coordinate of the mouse where the context menu was invoked.
  // Coords are relative to the associated RenderView's origin.
  //
  pub get_ycoord: Option<extern "C" fn(
      this: *mut cef_context_menu_params_t) -> libc::c_int>,

  //
  // Returns flags representing the type of node that the context menu was
  // invoked on.
  //
  pub get_type_flags: Option<extern "C" fn(
      this: *mut cef_context_menu_params_t) -> types::cef_context_menu_type_flags_t>,

  //
  // Returns the URL of the link, if any, that encloses the node that the
  // context menu was invoked on.
  //
  // The resulting string must be freed by calling cef_string_userfree_free().
  pub get_link_url: Option<extern "C" fn(
      this: *mut cef_context_menu_params_t) -> types::cef_string_userfree_t>,

  //
  // Returns the link URL, if any, to be used ONLY for "copy link address". We
  // don't validate this field in the frontend process.
  //
  // The resulting string must be freed by calling cef_string_userfree_free().
  pub get_unfiltered_link_url: Option<extern "C" fn(
      this: *mut cef_context_menu_params_t) -> types::cef_string_userfree_t>,

  //
  // Returns the source URL, if any, for the element that the context menu was
  // invoked on. Example of elements with source URLs are img, audio, and video.
  //
  // The resulting string must be freed by calling cef_string_userfree_free().
  pub get_source_url: Option<extern "C" fn(
      this: *mut cef_context_menu_params_t) -> types::cef_string_userfree_t>,

  //
  // Returns true (1) if the context menu was invoked on an image which has non-
  // NULL contents.
  //
  pub has_image_contents: Option<extern "C" fn(
      this: *mut cef_context_menu_params_t) -> libc::c_int>,

  //
  // Returns the URL of the top level page that the context menu was invoked on.
  //
  // The resulting string must be freed by calling cef_string_userfree_free().
  pub get_page_url: Option<extern "C" fn(
      this: *mut cef_context_menu_params_t) -> types::cef_string_userfree_t>,

  //
  // Returns the URL of the subframe that the context menu was invoked on.
  //
  // The resulting string must be freed by calling cef_string_userfree_free().
  pub get_frame_url: Option<extern "C" fn(
      this: *mut cef_context_menu_params_t) -> types::cef_string_userfree_t>,

  //
  // Returns the character encoding of the subframe that the context menu was
  // invoked on.
  //
  // The resulting string must be freed by calling cef_string_userfree_free().
  pub get_frame_charset: Option<extern "C" fn(
      this: *mut cef_context_menu_params_t) -> types::cef_string_userfree_t>,

  //
  // Returns the type of context node that the context menu was invoked on.
  //
  pub get_media_type: Option<extern "C" fn(
      this: *mut cef_context_menu_params_t) -> types::cef_context_menu_media_type_t>,

  //
  // Returns flags representing the actions supported by the media element, if
  // any, that the context menu was invoked on.
  //
  pub get_media_state_flags: Option<extern "C" fn(
      this: *mut cef_context_menu_params_t) -> types::cef_context_menu_media_state_flags_t>,

  //
  // Returns the text of the selection, if any, that the context menu was
  // invoked on.
  //
  // The resulting string must be freed by calling cef_string_userfree_free().
  pub get_selection_text: Option<extern "C" fn(
      this: *mut cef_context_menu_params_t) -> types::cef_string_userfree_t>,

  //
  // Returns the text of the misspelled word, if any, that the context menu was
  // invoked on.
  //
  // The resulting string must be freed by calling cef_string_userfree_free().
  pub get_misspelled_word: Option<extern "C" fn(
      this: *mut cef_context_menu_params_t) -> types::cef_string_userfree_t>,

  //
  // Returns the hash of the misspelled word, if any, that the context menu was
  // invoked on.
  //
  pub get_misspelling_hash: Option<extern "C" fn(
      this: *mut cef_context_menu_params_t) -> libc::c_int>,

  //
  // Returns true (1) if suggestions exist, false (0) otherwise. Fills in
  // |suggestions| from the spell check service for the misspelled word if there
  // is one.
  //
  pub get_dictionary_suggestions: Option<extern "C" fn(
      this: *mut cef_context_menu_params_t,
      suggestions: types::cef_string_list_t) -> libc::c_int>,

  //
  // Returns true (1) if the context menu was invoked on an editable node.
  //
  pub is_editable: Option<extern "C" fn(
      this: *mut cef_context_menu_params_t) -> libc::c_int>,

  //
  // Returns true (1) if the context menu was invoked on an editable node where
  // spell-check is enabled.
  //
  pub is_spell_check_enabled: Option<extern "C" fn(
      this: *mut cef_context_menu_params_t) -> libc::c_int>,

  //
  // Returns flags representing the actions supported by the editable node, if
  // any, that the context menu was invoked on.
  //
  pub get_edit_state_flags: Option<extern "C" fn(
      this: *mut cef_context_menu_params_t) -> types::cef_context_menu_edit_state_flags_t>,

  //
  // The reference count. This will only be present for Rust instances!
  //
  ref_count: uint,

  //
  // Extra data. This will only be present for Rust instances!
  //
  pub extra: u8,
} 

pub type cef_context_menu_params_t = _cef_context_menu_params_t;


//
// Provides information about the context menu state. The ethods of this
// structure can only be accessed on browser process the UI thread.
//
pub struct CefContextMenuParams {
  c_object: *mut cef_context_menu_params_t,
}

impl Clone for CefContextMenuParams {
  fn clone(&self) -> CefContextMenuParams{
    unsafe {
      if !self.c_object.is_null() {
        ((*self.c_object).base.add_ref.unwrap())(&mut (*self.c_object).base);
      }
      CefContextMenuParams {
        c_object: self.c_object,
      }
    }
  }
}

impl Drop for CefContextMenuParams {
  fn drop(&mut self) {
    unsafe {
      if !self.c_object.is_null() {
        ((*self.c_object).base.release.unwrap())(&mut (*self.c_object).base);
      }
    }
  }
}

impl CefContextMenuParams {
  pub unsafe fn from_c_object(c_object: *mut cef_context_menu_params_t) -> CefContextMenuParams {
    CefContextMenuParams {
      c_object: c_object,
    }
  }

  pub unsafe fn from_c_object_addref(c_object: *mut cef_context_menu_params_t) -> CefContextMenuParams {
    if !c_object.is_null() {
      ((*c_object).base.add_ref.unwrap())(&mut (*c_object).base);
    }
    CefContextMenuParams {
      c_object: c_object,
    }
  }

  pub fn c_object(&self) -> *mut cef_context_menu_params_t {
    self.c_object
  }

  pub fn c_object_addrefed(&self) -> *mut cef_context_menu_params_t {
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
  // Returns the X coordinate of the mouse where the context menu was invoked.
  // Coords are relative to the associated RenderView's origin.
  //
  pub fn get_xcoord(&self) -> libc::c_int {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).get_xcoord.unwrap())(
          self.c_object))
    }
  }

  //
  // Returns the Y coordinate of the mouse where the context menu was invoked.
  // Coords are relative to the associated RenderView's origin.
  //
  pub fn get_ycoord(&self) -> libc::c_int {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).get_ycoord.unwrap())(
          self.c_object))
    }
  }

  //
  // Returns flags representing the type of node that the context menu was
  // invoked on.
  //
  pub fn get_type_flags(&self) -> types::cef_context_menu_type_flags_t {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).get_type_flags.unwrap())(
          self.c_object))
    }
  }

  //
  // Returns the URL of the link, if any, that encloses the node that the
  // context menu was invoked on.
  //
  // The resulting string must be freed by calling cef_string_userfree_free().
  pub fn get_link_url(&self) -> String {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).get_link_url.unwrap())(
          self.c_object))
    }
  }

  //
  // Returns the link URL, if any, to be used ONLY for "copy link address". We
  // don't validate this field in the frontend process.
  //
  // The resulting string must be freed by calling cef_string_userfree_free().
  pub fn get_unfiltered_link_url(&self) -> String {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).get_unfiltered_link_url.unwrap())(
          self.c_object))
    }
  }

  //
  // Returns the source URL, if any, for the element that the context menu was
  // invoked on. Example of elements with source URLs are img, audio, and video.
  //
  // The resulting string must be freed by calling cef_string_userfree_free().
  pub fn get_source_url(&self) -> String {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).get_source_url.unwrap())(
          self.c_object))
    }
  }

  //
  // Returns true (1) if the context menu was invoked on an image which has non-
  // NULL contents.
  //
  pub fn has_image_contents(&self) -> libc::c_int {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).has_image_contents.unwrap())(
          self.c_object))
    }
  }

  //
  // Returns the URL of the top level page that the context menu was invoked on.
  //
  // The resulting string must be freed by calling cef_string_userfree_free().
  pub fn get_page_url(&self) -> String {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).get_page_url.unwrap())(
          self.c_object))
    }
  }

  //
  // Returns the URL of the subframe that the context menu was invoked on.
  //
  // The resulting string must be freed by calling cef_string_userfree_free().
  pub fn get_frame_url(&self) -> String {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).get_frame_url.unwrap())(
          self.c_object))
    }
  }

  //
  // Returns the character encoding of the subframe that the context menu was
  // invoked on.
  //
  // The resulting string must be freed by calling cef_string_userfree_free().
  pub fn get_frame_charset(&self) -> String {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).get_frame_charset.unwrap())(
          self.c_object))
    }
  }

  //
  // Returns the type of context node that the context menu was invoked on.
  //
  pub fn get_media_type(&self) -> types::cef_context_menu_media_type_t {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).get_media_type.unwrap())(
          self.c_object))
    }
  }

  //
  // Returns flags representing the actions supported by the media element, if
  // any, that the context menu was invoked on.
  //
  pub fn get_media_state_flags(
      &self) -> types::cef_context_menu_media_state_flags_t {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).get_media_state_flags.unwrap())(
          self.c_object))
    }
  }

  //
  // Returns the text of the selection, if any, that the context menu was
  // invoked on.
  //
  // The resulting string must be freed by calling cef_string_userfree_free().
  pub fn get_selection_text(&self) -> String {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).get_selection_text.unwrap())(
          self.c_object))
    }
  }

  //
  // Returns the text of the misspelled word, if any, that the context menu was
  // invoked on.
  //
  // The resulting string must be freed by calling cef_string_userfree_free().
  pub fn get_misspelled_word(&self) -> String {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).get_misspelled_word.unwrap())(
          self.c_object))
    }
  }

  //
  // Returns the hash of the misspelled word, if any, that the context menu was
  // invoked on.
  //
  pub fn get_misspelling_hash(&self) -> libc::c_int {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).get_misspelling_hash.unwrap())(
          self.c_object))
    }
  }

  //
  // Returns true (1) if suggestions exist, false (0) otherwise. Fills in
  // |suggestions| from the spell check service for the misspelled word if there
  // is one.
  //
  pub fn get_dictionary_suggestions(&self,
      suggestions: Vec<String>) -> libc::c_int {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).get_dictionary_suggestions.unwrap())(
          self.c_object,
          CefWrap::to_c(suggestions)))
    }
  }

  //
  // Returns true (1) if the context menu was invoked on an editable node.
  //
  pub fn is_editable(&self) -> libc::c_int {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).is_editable.unwrap())(
          self.c_object))
    }
  }

  //
  // Returns true (1) if the context menu was invoked on an editable node where
  // spell-check is enabled.
  //
  pub fn is_spell_check_enabled(&self) -> libc::c_int {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).is_spell_check_enabled.unwrap())(
          self.c_object))
    }
  }

  //
  // Returns flags representing the actions supported by the editable node, if
  // any, that the context menu was invoked on.
  //
  pub fn get_edit_state_flags(
      &self) -> types::cef_context_menu_edit_state_flags_t {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).get_edit_state_flags.unwrap())(
          self.c_object))
    }
  }
} 

impl CefWrap<*mut cef_context_menu_params_t> for CefContextMenuParams {
  fn to_c(rust_object: CefContextMenuParams) -> *mut cef_context_menu_params_t {
    rust_object.c_object_addrefed()
  }
  unsafe fn to_rust(c_object: *mut cef_context_menu_params_t) -> CefContextMenuParams {
    CefContextMenuParams::from_c_object_addref(c_object)
  }
}
impl CefWrap<*mut cef_context_menu_params_t> for Option<CefContextMenuParams> {
  fn to_c(rust_object: Option<CefContextMenuParams>) -> *mut cef_context_menu_params_t {
    match rust_object {
      None => ptr::null_mut(),
      Some(rust_object) => rust_object.c_object_addrefed(),
    }
  }
  unsafe fn to_rust(c_object: *mut cef_context_menu_params_t) -> Option<CefContextMenuParams> {
    if c_object.is_null() {
      None
    } else {
      Some(CefContextMenuParams::from_c_object_addref(c_object))
    }
  }
}

