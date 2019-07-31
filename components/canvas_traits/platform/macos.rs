/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::platform::NativeSurfaceFormat;
use euclid::default::Size2D;
use gleam::gl::{self, Gl, GLuint};
use io_surface::{self, IOSurface};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::{self, Debug, Formatter};

#[derive(Clone)]
pub struct SerializableIOSurface(IOSurface);

// FIXME(pcwalton): We should turn the IOSurface into a Mach port instead of using global IDs.
impl Serialize for SerializableIOSurface {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        serializer.serialize_u32(self.0.get_id())
    }
}

// FIXME(pcwalton): We should turn the IOSurface into a Mach port instead of using global IDs.
impl<'de> Deserialize<'de> for SerializableIOSurface {
    fn deserialize<D>(d: D) -> Result<SerializableIOSurface, D::Error> where D: Deserializer<'de> {
        Ok(SerializableIOSurface(io_surface::lookup(Deserialize::deserialize(d)?)))
    }
}

#[allow(unsafe_code)]
unsafe impl Send for SerializableIOSurface {}

#[derive(Clone, Serialize, Deserialize)]
pub struct NativeSurface {
    io_surface: SerializableIOSurface,
    size: Size2D<i32>,
    format: NativeSurfaceFormat,
}

#[derive(Clone)]
pub struct NativeSurfaceTexture {
    io_surface: Option<IOSurface>,
    size: Size2D<i32>,
    format: NativeSurfaceFormat,
    gl_texture: GLuint,
}

impl Debug for NativeSurface {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        write!(f, "{:?}, {:?}", self.size, self.format)
    }
}

impl Drop for NativeSurfaceTexture {
    fn drop(&mut self) {
        debug_assert!(false, "Must destroy the native surface binder manually!");
    }
}

impl NativeSurface {
    #[inline]
    pub fn size(&self) -> Size2D<i32> {
        self.size
    }

    #[inline]
    pub fn format(&self) -> NativeSurfaceFormat {
        self.format
    }
}

impl NativeSurfaceTexture {
    #[inline]
    pub fn new(gl: &dyn gl::Gl, size: &Size2D<i32>, format: NativeSurfaceFormat)
               -> NativeSurfaceTexture {
        let mut gl_texture = 0;
        let gl_texture = gl.gen_textures(1)[0];
        NativeSurfaceTexture { size: *size, format, io_surface: None, gl_texture }
    }

    pub fn bind(&mut self, gl: &dyn Gl, surface: NativeSurface) -> Result<(), NativeSurface> {
        if surface.size != self.size || surface.format != self.format {
            return Err(surface)
        }

        let has_alpha = match self.format {
            NativeSurfaceFormat::Rgb => false,
            NativeSurfaceFormat::Rgba => true,
        };
        gl.bind_texture(gl::TEXTURE_RECTANGLE_ARB, self.gl_texture);
        surface.io_surface.0.bind_to_gl_texture(self.size.width, self.size.height, has_alpha);
        gl.bind_texture(gl::TEXTURE_RECTANGLE_ARB, 0);
        self.io_surface = Some(surface.io_surface.0);
        Ok(())
    }

    pub fn destroy(&mut self, gl: &dyn Gl) {
        gl.delete_textures(&[self.gl_texture]);
    }

    #[inline]
    pub fn gl_texture(&self) -> GLuint {
        self.gl_texture
    }

    #[inline]
    pub fn size(&self) -> Size2D<i32> {
        self.size
    }
}
