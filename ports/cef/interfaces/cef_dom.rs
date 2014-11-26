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
// Structure to implement for visiting the DOM. The functions of this structure
// will be called on the render process main thread.
//
#[repr(C)]
pub struct _cef_domvisitor_t {
  //
  // Base structure.
  //
  pub base: types::cef_base_t,

  //
  // Method executed for visiting the DOM. The document object passed to this
  // function represents a snapshot of the DOM at the time this function is
  // executed. DOM objects are only valid for the scope of this function. Do not
  // keep references to or attempt to access any DOM objects outside the scope
  // of this function.
  //
  pub visit: Option<extern "C" fn(this: *mut cef_domvisitor_t,
      document: *mut interfaces::cef_domdocument_t) -> ()>,

  //
  // The reference count. This will only be present for Rust instances!
  //
  ref_count: uint,

  //
  // Extra data. This will only be present for Rust instances!
  //
  pub extra: u8,
} 

pub type cef_domvisitor_t = _cef_domvisitor_t;


//
// Structure to implement for visiting the DOM. The functions of this structure
// will be called on the render process main thread.
//
pub struct CefDOMVisitor {
  c_object: *mut cef_domvisitor_t,
}

impl Clone for CefDOMVisitor {
  fn clone(&self) -> CefDOMVisitor{
    unsafe {
      if !self.c_object.is_null() {
        ((*self.c_object).base.add_ref.unwrap())(&mut (*self.c_object).base);
      }
      CefDOMVisitor {
        c_object: self.c_object,
      }
    }
  }
}

impl Drop for CefDOMVisitor {
  fn drop(&mut self) {
    unsafe {
      if !self.c_object.is_null() {
        ((*self.c_object).base.release.unwrap())(&mut (*self.c_object).base);
      }
    }
  }
}

impl CefDOMVisitor {
  pub unsafe fn from_c_object(c_object: *mut cef_domvisitor_t) -> CefDOMVisitor {
    CefDOMVisitor {
      c_object: c_object,
    }
  }

  pub unsafe fn from_c_object_addref(c_object: *mut cef_domvisitor_t) -> CefDOMVisitor {
    if !c_object.is_null() {
      ((*c_object).base.add_ref.unwrap())(&mut (*c_object).base);
    }
    CefDOMVisitor {
      c_object: c_object,
    }
  }

  pub fn c_object(&self) -> *mut cef_domvisitor_t {
    self.c_object
  }

  pub fn c_object_addrefed(&self) -> *mut cef_domvisitor_t {
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
  // Method executed for visiting the DOM. The document object passed to this
  // function represents a snapshot of the DOM at the time this function is
  // executed. DOM objects are only valid for the scope of this function. Do not
  // keep references to or attempt to access any DOM objects outside the scope
  // of this function.
  //
  pub fn visit(&self, document: interfaces::CefDOMDocument) -> () {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).visit.unwrap())(
          self.c_object,
          CefWrap::to_c(document)))
    }
  }
} 

impl CefWrap<*mut cef_domvisitor_t> for CefDOMVisitor {
  fn to_c(rust_object: CefDOMVisitor) -> *mut cef_domvisitor_t {
    rust_object.c_object_addrefed()
  }
  unsafe fn to_rust(c_object: *mut cef_domvisitor_t) -> CefDOMVisitor {
    CefDOMVisitor::from_c_object_addref(c_object)
  }
}
impl CefWrap<*mut cef_domvisitor_t> for Option<CefDOMVisitor> {
  fn to_c(rust_object: Option<CefDOMVisitor>) -> *mut cef_domvisitor_t {
    match rust_object {
      None => ptr::null_mut(),
      Some(rust_object) => rust_object.c_object_addrefed(),
    }
  }
  unsafe fn to_rust(c_object: *mut cef_domvisitor_t) -> Option<CefDOMVisitor> {
    if c_object.is_null() {
      None
    } else {
      Some(CefDOMVisitor::from_c_object_addref(c_object))
    }
  }
}


//
// Structure used to represent a DOM document. The functions of this structure
// should only be called on the render process main thread thread.
//
#[repr(C)]
pub struct _cef_domdocument_t {
  //
  // Base structure.
  //
  pub base: types::cef_base_t,

  //
  // Returns the document type.
  //
  pub get_type: Option<extern "C" fn(
      this: *mut cef_domdocument_t) -> types::cef_dom_document_type_t>,

  //
  // Returns the root document node.
  //
  pub get_document: Option<extern "C" fn(
      this: *mut cef_domdocument_t) -> *mut interfaces::cef_domnode_t>,

  //
  // Returns the BODY node of an HTML document.
  //
  pub get_body: Option<extern "C" fn(
      this: *mut cef_domdocument_t) -> *mut interfaces::cef_domnode_t>,

  //
  // Returns the HEAD node of an HTML document.
  //
  pub get_head: Option<extern "C" fn(
      this: *mut cef_domdocument_t) -> *mut interfaces::cef_domnode_t>,

  //
  // Returns the title of an HTML document.
  //
  // The resulting string must be freed by calling cef_string_userfree_free().
  pub get_title: Option<extern "C" fn(
      this: *mut cef_domdocument_t) -> types::cef_string_userfree_t>,

  //
  // Returns the document element with the specified ID value.
  //
  pub get_element_by_id: Option<extern "C" fn(this: *mut cef_domdocument_t,
      id: *const types::cef_string_t) -> *mut interfaces::cef_domnode_t>,

  //
  // Returns the node that currently has keyboard focus.
  //
  pub get_focused_node: Option<extern "C" fn(
      this: *mut cef_domdocument_t) -> *mut interfaces::cef_domnode_t>,

  //
  // Returns true (1) if a portion of the document is selected.
  //
  pub has_selection: Option<extern "C" fn(
      this: *mut cef_domdocument_t) -> libc::c_int>,

  //
  // Returns the selection start node.
  //
  pub get_selection_start_node: Option<extern "C" fn(
      this: *mut cef_domdocument_t) -> *mut interfaces::cef_domnode_t>,

  //
  // Returns the selection offset within the start node.
  //
  pub get_selection_start_offset: Option<extern "C" fn(
      this: *mut cef_domdocument_t) -> libc::c_int>,

  //
  // Returns the selection end node.
  //
  pub get_selection_end_node: Option<extern "C" fn(
      this: *mut cef_domdocument_t) -> *mut interfaces::cef_domnode_t>,

  //
  // Returns the selection offset within the end node.
  //
  pub get_selection_end_offset: Option<extern "C" fn(
      this: *mut cef_domdocument_t) -> libc::c_int>,

  //
  // Returns the contents of this selection as markup.
  //
  // The resulting string must be freed by calling cef_string_userfree_free().
  pub get_selection_as_markup: Option<extern "C" fn(
      this: *mut cef_domdocument_t) -> types::cef_string_userfree_t>,

  //
  // Returns the contents of this selection as text.
  //
  // The resulting string must be freed by calling cef_string_userfree_free().
  pub get_selection_as_text: Option<extern "C" fn(
      this: *mut cef_domdocument_t) -> types::cef_string_userfree_t>,

  //
  // Returns the base URL for the document.
  //
  // The resulting string must be freed by calling cef_string_userfree_free().
  pub get_base_url: Option<extern "C" fn(
      this: *mut cef_domdocument_t) -> types::cef_string_userfree_t>,

  //
  // Returns a complete URL based on the document base URL and the specified
  // partial URL.
  //
  // The resulting string must be freed by calling cef_string_userfree_free().
  pub get_complete_url: Option<extern "C" fn(this: *mut cef_domdocument_t,
      partialURL: *const types::cef_string_t) -> types::cef_string_userfree_t>,

  //
  // The reference count. This will only be present for Rust instances!
  //
  ref_count: uint,

  //
  // Extra data. This will only be present for Rust instances!
  //
  pub extra: u8,
} 

pub type cef_domdocument_t = _cef_domdocument_t;


//
// Structure used to represent a DOM document. The functions of this structure
// should only be called on the render process main thread thread.
//
pub struct CefDOMDocument {
  c_object: *mut cef_domdocument_t,
}

impl Clone for CefDOMDocument {
  fn clone(&self) -> CefDOMDocument{
    unsafe {
      if !self.c_object.is_null() {
        ((*self.c_object).base.add_ref.unwrap())(&mut (*self.c_object).base);
      }
      CefDOMDocument {
        c_object: self.c_object,
      }
    }
  }
}

impl Drop for CefDOMDocument {
  fn drop(&mut self) {
    unsafe {
      if !self.c_object.is_null() {
        ((*self.c_object).base.release.unwrap())(&mut (*self.c_object).base);
      }
    }
  }
}

impl CefDOMDocument {
  pub unsafe fn from_c_object(c_object: *mut cef_domdocument_t) -> CefDOMDocument {
    CefDOMDocument {
      c_object: c_object,
    }
  }

  pub unsafe fn from_c_object_addref(c_object: *mut cef_domdocument_t) -> CefDOMDocument {
    if !c_object.is_null() {
      ((*c_object).base.add_ref.unwrap())(&mut (*c_object).base);
    }
    CefDOMDocument {
      c_object: c_object,
    }
  }

  pub fn c_object(&self) -> *mut cef_domdocument_t {
    self.c_object
  }

  pub fn c_object_addrefed(&self) -> *mut cef_domdocument_t {
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
  // Returns the document type.
  //
  pub fn get_type(&self) -> types::cef_dom_document_type_t {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).get_type.unwrap())(
          self.c_object))
    }
  }

  //
  // Returns the root document node.
  //
  pub fn get_document(&self) -> interfaces::CefDOMNode {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).get_document.unwrap())(
          self.c_object))
    }
  }

  //
  // Returns the BODY node of an HTML document.
  //
  pub fn get_body(&self) -> interfaces::CefDOMNode {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).get_body.unwrap())(
          self.c_object))
    }
  }

  //
  // Returns the HEAD node of an HTML document.
  //
  pub fn get_head(&self) -> interfaces::CefDOMNode {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).get_head.unwrap())(
          self.c_object))
    }
  }

  //
  // Returns the title of an HTML document.
  //
  // The resulting string must be freed by calling cef_string_userfree_free().
  pub fn get_title(&self) -> String {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).get_title.unwrap())(
          self.c_object))
    }
  }

  //
  // Returns the document element with the specified ID value.
  //
  pub fn get_element_by_id(&self, id: &[u16]) -> interfaces::CefDOMNode {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).get_element_by_id.unwrap())(
          self.c_object,
          CefWrap::to_c(id)))
    }
  }

  //
  // Returns the node that currently has keyboard focus.
  //
  pub fn get_focused_node(&self) -> interfaces::CefDOMNode {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).get_focused_node.unwrap())(
          self.c_object))
    }
  }

  //
  // Returns true (1) if a portion of the document is selected.
  //
  pub fn has_selection(&self) -> libc::c_int {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).has_selection.unwrap())(
          self.c_object))
    }
  }

  //
  // Returns the selection start node.
  //
  pub fn get_selection_start_node(&self) -> interfaces::CefDOMNode {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).get_selection_start_node.unwrap())(
          self.c_object))
    }
  }

  //
  // Returns the selection offset within the start node.
  //
  pub fn get_selection_start_offset(&self) -> libc::c_int {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).get_selection_start_offset.unwrap())(
          self.c_object))
    }
  }

  //
  // Returns the selection end node.
  //
  pub fn get_selection_end_node(&self) -> interfaces::CefDOMNode {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).get_selection_end_node.unwrap())(
          self.c_object))
    }
  }

  //
  // Returns the selection offset within the end node.
  //
  pub fn get_selection_end_offset(&self) -> libc::c_int {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).get_selection_end_offset.unwrap())(
          self.c_object))
    }
  }

  //
  // Returns the contents of this selection as markup.
  //
  // The resulting string must be freed by calling cef_string_userfree_free().
  pub fn get_selection_as_markup(&self) -> String {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).get_selection_as_markup.unwrap())(
          self.c_object))
    }
  }

  //
  // Returns the contents of this selection as text.
  //
  // The resulting string must be freed by calling cef_string_userfree_free().
  pub fn get_selection_as_text(&self) -> String {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).get_selection_as_text.unwrap())(
          self.c_object))
    }
  }

  //
  // Returns the base URL for the document.
  //
  // The resulting string must be freed by calling cef_string_userfree_free().
  pub fn get_base_url(&self) -> String {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).get_base_url.unwrap())(
          self.c_object))
    }
  }

  //
  // Returns a complete URL based on the document base URL and the specified
  // partial URL.
  //
  // The resulting string must be freed by calling cef_string_userfree_free().
  pub fn get_complete_url(&self, partialURL: &[u16]) -> String {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).get_complete_url.unwrap())(
          self.c_object,
          CefWrap::to_c(partialURL)))
    }
  }
} 

impl CefWrap<*mut cef_domdocument_t> for CefDOMDocument {
  fn to_c(rust_object: CefDOMDocument) -> *mut cef_domdocument_t {
    rust_object.c_object_addrefed()
  }
  unsafe fn to_rust(c_object: *mut cef_domdocument_t) -> CefDOMDocument {
    CefDOMDocument::from_c_object_addref(c_object)
  }
}
impl CefWrap<*mut cef_domdocument_t> for Option<CefDOMDocument> {
  fn to_c(rust_object: Option<CefDOMDocument>) -> *mut cef_domdocument_t {
    match rust_object {
      None => ptr::null_mut(),
      Some(rust_object) => rust_object.c_object_addrefed(),
    }
  }
  unsafe fn to_rust(c_object: *mut cef_domdocument_t) -> Option<CefDOMDocument> {
    if c_object.is_null() {
      None
    } else {
      Some(CefDOMDocument::from_c_object_addref(c_object))
    }
  }
}


//
// Structure used to represent a DOM node. The functions of this structure
// should only be called on the render process main thread.
//
#[repr(C)]
pub struct _cef_domnode_t {
  //
  // Base structure.
  //
  pub base: types::cef_base_t,

  //
  // Returns the type for this node.
  //
  pub get_type: Option<extern "C" fn(
      this: *mut cef_domnode_t) -> types::cef_dom_node_type_t>,

  //
  // Returns true (1) if this is a text node.
  //
  pub is_text: Option<extern "C" fn(this: *mut cef_domnode_t) -> libc::c_int>,

  //
  // Returns true (1) if this is an element node.
  //
  pub is_element: Option<extern "C" fn(
      this: *mut cef_domnode_t) -> libc::c_int>,

  //
  // Returns true (1) if this is an editable node.
  //
  pub is_editable: Option<extern "C" fn(
      this: *mut cef_domnode_t) -> libc::c_int>,

  //
  // Returns true (1) if this is a form control element node.
  //
  pub is_form_control_element: Option<extern "C" fn(
      this: *mut cef_domnode_t) -> libc::c_int>,

  //
  // Returns the type of this form control element node.
  //
  // The resulting string must be freed by calling cef_string_userfree_free().
  pub get_form_control_element_type: Option<extern "C" fn(
      this: *mut cef_domnode_t) -> types::cef_string_userfree_t>,

  //
  // Returns true (1) if this object is pointing to the same handle as |that|
  // object.
  //
  pub is_same: Option<extern "C" fn(this: *mut cef_domnode_t,
      that: *mut interfaces::cef_domnode_t) -> libc::c_int>,

  //
  // Returns the name of this node.
  //
  // The resulting string must be freed by calling cef_string_userfree_free().
  pub get_name: Option<extern "C" fn(
      this: *mut cef_domnode_t) -> types::cef_string_userfree_t>,

  //
  // Returns the value of this node.
  //
  // The resulting string must be freed by calling cef_string_userfree_free().
  pub get_value: Option<extern "C" fn(
      this: *mut cef_domnode_t) -> types::cef_string_userfree_t>,

  //
  // Set the value of this node. Returns true (1) on success.
  //
  pub set_value: Option<extern "C" fn(this: *mut cef_domnode_t,
      value: *const types::cef_string_t) -> libc::c_int>,

  //
  // Returns the contents of this node as markup.
  //
  // The resulting string must be freed by calling cef_string_userfree_free().
  pub get_as_markup: Option<extern "C" fn(
      this: *mut cef_domnode_t) -> types::cef_string_userfree_t>,

  //
  // Returns the document associated with this node.
  //
  pub get_document: Option<extern "C" fn(
      this: *mut cef_domnode_t) -> *mut interfaces::cef_domdocument_t>,

  //
  // Returns the parent node.
  //
  pub get_parent: Option<extern "C" fn(
      this: *mut cef_domnode_t) -> *mut interfaces::cef_domnode_t>,

  //
  // Returns the previous sibling node.
  //
  pub get_previous_sibling: Option<extern "C" fn(
      this: *mut cef_domnode_t) -> *mut interfaces::cef_domnode_t>,

  //
  // Returns the next sibling node.
  //
  pub get_next_sibling: Option<extern "C" fn(
      this: *mut cef_domnode_t) -> *mut interfaces::cef_domnode_t>,

  //
  // Returns true (1) if this node has child nodes.
  //
  pub has_children: Option<extern "C" fn(
      this: *mut cef_domnode_t) -> libc::c_int>,

  //
  // Return the first child node.
  //
  pub get_first_child: Option<extern "C" fn(
      this: *mut cef_domnode_t) -> *mut interfaces::cef_domnode_t>,

  //
  // Returns the last child node.
  //
  pub get_last_child: Option<extern "C" fn(
      this: *mut cef_domnode_t) -> *mut interfaces::cef_domnode_t>,


  // The following functions are valid only for element nodes.

  //
  // Returns the tag name of this element.
  //
  // The resulting string must be freed by calling cef_string_userfree_free().
  pub get_element_tag_name: Option<extern "C" fn(
      this: *mut cef_domnode_t) -> types::cef_string_userfree_t>,

  //
  // Returns true (1) if this element has attributes.
  //
  pub has_element_attributes: Option<extern "C" fn(
      this: *mut cef_domnode_t) -> libc::c_int>,

  //
  // Returns true (1) if this element has an attribute named |attrName|.
  //
  pub has_element_attribute: Option<extern "C" fn(this: *mut cef_domnode_t,
      attrName: *const types::cef_string_t) -> libc::c_int>,

  //
  // Returns the element attribute named |attrName|.
  //
  // The resulting string must be freed by calling cef_string_userfree_free().
  pub get_element_attribute: Option<extern "C" fn(this: *mut cef_domnode_t,
      attrName: *const types::cef_string_t) -> types::cef_string_userfree_t>,

  //
  // Returns a map of all element attributes.
  //
  pub get_element_attributes: Option<extern "C" fn(this: *mut cef_domnode_t,
      attrMap: types::cef_string_map_t) -> ()>,

  //
  // Set the value for the element attribute named |attrName|. Returns true (1)
  // on success.
  //
  pub set_element_attribute: Option<extern "C" fn(this: *mut cef_domnode_t,
      attrName: *const types::cef_string_t,
      value: *const types::cef_string_t) -> libc::c_int>,

  //
  // Returns the inner text of the element.
  //
  // The resulting string must be freed by calling cef_string_userfree_free().
  pub get_element_inner_text: Option<extern "C" fn(
      this: *mut cef_domnode_t) -> types::cef_string_userfree_t>,

  //
  // The reference count. This will only be present for Rust instances!
  //
  ref_count: uint,

  //
  // Extra data. This will only be present for Rust instances!
  //
  pub extra: u8,
} 

pub type cef_domnode_t = _cef_domnode_t;


//
// Structure used to represent a DOM node. The functions of this structure
// should only be called on the render process main thread.
//
pub struct CefDOMNode {
  c_object: *mut cef_domnode_t,
}

impl Clone for CefDOMNode {
  fn clone(&self) -> CefDOMNode{
    unsafe {
      if !self.c_object.is_null() {
        ((*self.c_object).base.add_ref.unwrap())(&mut (*self.c_object).base);
      }
      CefDOMNode {
        c_object: self.c_object,
      }
    }
  }
}

impl Drop for CefDOMNode {
  fn drop(&mut self) {
    unsafe {
      if !self.c_object.is_null() {
        ((*self.c_object).base.release.unwrap())(&mut (*self.c_object).base);
      }
    }
  }
}

impl CefDOMNode {
  pub unsafe fn from_c_object(c_object: *mut cef_domnode_t) -> CefDOMNode {
    CefDOMNode {
      c_object: c_object,
    }
  }

  pub unsafe fn from_c_object_addref(c_object: *mut cef_domnode_t) -> CefDOMNode {
    if !c_object.is_null() {
      ((*c_object).base.add_ref.unwrap())(&mut (*c_object).base);
    }
    CefDOMNode {
      c_object: c_object,
    }
  }

  pub fn c_object(&self) -> *mut cef_domnode_t {
    self.c_object
  }

  pub fn c_object_addrefed(&self) -> *mut cef_domnode_t {
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
  // Returns the type for this node.
  //
  pub fn get_type(&self) -> types::cef_dom_node_type_t {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).get_type.unwrap())(
          self.c_object))
    }
  }

  //
  // Returns true (1) if this is a text node.
  //
  pub fn is_text(&self) -> libc::c_int {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).is_text.unwrap())(
          self.c_object))
    }
  }

  //
  // Returns true (1) if this is an element node.
  //
  pub fn is_element(&self) -> libc::c_int {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).is_element.unwrap())(
          self.c_object))
    }
  }

  //
  // Returns true (1) if this is an editable node.
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
  // Returns true (1) if this is a form control element node.
  //
  pub fn is_form_control_element(&self) -> libc::c_int {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).is_form_control_element.unwrap())(
          self.c_object))
    }
  }

  //
  // Returns the type of this form control element node.
  //
  // The resulting string must be freed by calling cef_string_userfree_free().
  pub fn get_form_control_element_type(&self) -> String {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).get_form_control_element_type.unwrap())(
          self.c_object))
    }
  }

  //
  // Returns true (1) if this object is pointing to the same handle as |that|
  // object.
  //
  pub fn is_same(&self, that: interfaces::CefDOMNode) -> libc::c_int {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).is_same.unwrap())(
          self.c_object,
          CefWrap::to_c(that)))
    }
  }

  //
  // Returns the name of this node.
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
  // Returns the value of this node.
  //
  // The resulting string must be freed by calling cef_string_userfree_free().
  pub fn get_value(&self) -> String {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).get_value.unwrap())(
          self.c_object))
    }
  }

  //
  // Set the value of this node. Returns true (1) on success.
  //
  pub fn set_value(&self, value: &[u16]) -> libc::c_int {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).set_value.unwrap())(
          self.c_object,
          CefWrap::to_c(value)))
    }
  }

  //
  // Returns the contents of this node as markup.
  //
  // The resulting string must be freed by calling cef_string_userfree_free().
  pub fn get_as_markup(&self) -> String {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).get_as_markup.unwrap())(
          self.c_object))
    }
  }

  //
  // Returns the document associated with this node.
  //
  pub fn get_document(&self) -> interfaces::CefDOMDocument {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).get_document.unwrap())(
          self.c_object))
    }
  }

  //
  // Returns the parent node.
  //
  pub fn get_parent(&self) -> interfaces::CefDOMNode {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).get_parent.unwrap())(
          self.c_object))
    }
  }

  //
  // Returns the previous sibling node.
  //
  pub fn get_previous_sibling(&self) -> interfaces::CefDOMNode {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).get_previous_sibling.unwrap())(
          self.c_object))
    }
  }

  //
  // Returns the next sibling node.
  //
  pub fn get_next_sibling(&self) -> interfaces::CefDOMNode {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).get_next_sibling.unwrap())(
          self.c_object))
    }
  }

  //
  // Returns true (1) if this node has child nodes.
  //
  pub fn has_children(&self) -> libc::c_int {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).has_children.unwrap())(
          self.c_object))
    }
  }

  //
  // Return the first child node.
  //
  pub fn get_first_child(&self) -> interfaces::CefDOMNode {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).get_first_child.unwrap())(
          self.c_object))
    }
  }

  //
  // Returns the last child node.
  //
  pub fn get_last_child(&self) -> interfaces::CefDOMNode {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).get_last_child.unwrap())(
          self.c_object))
    }
  }


  // The following functions are valid only for element nodes.

  //
  // Returns the tag name of this element.
  //
  // The resulting string must be freed by calling cef_string_userfree_free().
  pub fn get_element_tag_name(&self) -> String {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).get_element_tag_name.unwrap())(
          self.c_object))
    }
  }

  //
  // Returns true (1) if this element has attributes.
  //
  pub fn has_element_attributes(&self) -> libc::c_int {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).has_element_attributes.unwrap())(
          self.c_object))
    }
  }

  //
  // Returns true (1) if this element has an attribute named |attrName|.
  //
  pub fn has_element_attribute(&self, attrName: &[u16]) -> libc::c_int {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).has_element_attribute.unwrap())(
          self.c_object,
          CefWrap::to_c(attrName)))
    }
  }

  //
  // Returns the element attribute named |attrName|.
  //
  // The resulting string must be freed by calling cef_string_userfree_free().
  pub fn get_element_attribute(&self, attrName: &[u16]) -> String {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).get_element_attribute.unwrap())(
          self.c_object,
          CefWrap::to_c(attrName)))
    }
  }

  //
  // Returns a map of all element attributes.
  //
  pub fn get_element_attributes(&self, attrMap: HashMap<String,String>) -> () {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).get_element_attributes.unwrap())(
          self.c_object,
          CefWrap::to_c(attrMap)))
    }
  }

  //
  // Set the value for the element attribute named |attrName|. Returns true (1)
  // on success.
  //
  pub fn set_element_attribute(&self, attrName: &[u16],
      value: &[u16]) -> libc::c_int {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).set_element_attribute.unwrap())(
          self.c_object,
          CefWrap::to_c(attrName),
          CefWrap::to_c(value)))
    }
  }

  //
  // Returns the inner text of the element.
  //
  // The resulting string must be freed by calling cef_string_userfree_free().
  pub fn get_element_inner_text(&self) -> String {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).get_element_inner_text.unwrap())(
          self.c_object))
    }
  }
} 

impl CefWrap<*mut cef_domnode_t> for CefDOMNode {
  fn to_c(rust_object: CefDOMNode) -> *mut cef_domnode_t {
    rust_object.c_object_addrefed()
  }
  unsafe fn to_rust(c_object: *mut cef_domnode_t) -> CefDOMNode {
    CefDOMNode::from_c_object_addref(c_object)
  }
}
impl CefWrap<*mut cef_domnode_t> for Option<CefDOMNode> {
  fn to_c(rust_object: Option<CefDOMNode>) -> *mut cef_domnode_t {
    match rust_object {
      None => ptr::null_mut(),
      Some(rust_object) => rust_object.c_object_addrefed(),
    }
  }
  unsafe fn to_rust(c_object: *mut cef_domnode_t) -> Option<CefDOMNode> {
    if c_object.is_null() {
      None
    } else {
      Some(CefDOMNode::from_c_object_addref(c_object))
    }
  }
}

