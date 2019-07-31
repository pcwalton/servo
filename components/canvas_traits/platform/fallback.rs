/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::platform::NativeSurfaceFormat;
use euclid::default::Size2D;
use gleam::gl::{self, Gl, GLsync, GLuint};
use std::fmt::{self, Debug, Formatter};

#[derive(Clone)]
pub struct NativeSurface {
    texture: GLuint,
    sync: GLsync,
    size: Size2D<i32>,
    format: NativeSurfaceFormat,
}

#[derive(Clone)]
pub struct NativeSurfaceTexture {
    texture: GLuint,
    size: Size2D<i32>,
    format: NativeSurfaceFormat,
}

impl Debug for NativeSurface {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        write!(f, "{:?} {:?}, {:?}", self.texture, self.size, self.format)
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
        NativeSurfaceTexture { texture: 0, size: *size, format }
    }

    pub fn bind(&mut self, gl: &dyn Gl, surface: NativeSurface) -> Result<(), NativeSurface> {
        if surface.size != self.size || surface.format != self.format {
            return Err(surface)
        }

        // Make sure the texture is ready, then attach the texture.
        gl.wait_sync(surface.sync, 0, gl::TIMEOUT_IGNORED);
        self.texture = surface.texture;
        Ok(())
    }

    pub fn destroy(&mut self, gl: &dyn Gl) {
        // We don't own the texture, so don't destroy it.
    }

    #[inline]
    pub fn gl_texture(&self) -> GLuint {
        self.texture
    }

    #[inline]
    pub fn size(&self) -> Size2D<i32> {
        self.size
    }
}
