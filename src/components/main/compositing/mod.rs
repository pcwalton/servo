/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

pub use windowing;

use windowing::WindowMethods;

use azure::azure_hl::SourceSurfaceMethods;
use gfx::opts::Opts;
use layers::platform::surface::NativeCompositingGraphicsContext;
use servo_msg::compositor_msg::{CompositorComm, Tile};
use servo_msg::constellation_msg::{RenderListener, ScriptListener};
use servo_util::time::ProfilerChan;
use std::num::Orderable;

#[cfg(target_os="linux")]
use azure::azure_hl;

mod quadtree;
mod compositor_layer;

mod run;
mod run_headless;

pub struct CompositorTask {
    opts: Opts,
    comm: CompositorComm,
    profiler_chan: ProfilerChan,
}

impl CompositorTask {
    pub fn new(opts: Opts, comm: CompositorComm, profiler_chan: ProfilerChan) -> CompositorTask {
        CompositorTask {
            opts: opts,
            comm: comm,
            profiler_chan: profiler_chan,
        }
    }

    /// Creates a graphics context. Platform-specific.
    ///
    /// FIXME(pcwalton): Probably could be less platform-specific, using the metadata abstraction.
    #[cfg(target_os="linux")]
    fn create_graphics_context() -> NativeCompositingGraphicsContext {
        NativeCompositingGraphicsContext::from_display(azure_hl::current_display())
    }
    #[cfg(not(target_os="linux"))]
    fn create_graphics_context() -> NativeCompositingGraphicsContext {
        NativeCompositingGraphicsContext::new()
    }

    pub fn run(&mut self) {
        if self.opts.headless {
            run_headless::run_compositor(self);
        } else {
            run::run_compositor(self);
        }
    }
}
