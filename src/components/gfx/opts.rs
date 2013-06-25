/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Configuration options for a single run of the Servo application. Created
//! from command line arguments.

use azure::azure_hl::{BackendType, CairoBackend, CoreGraphicsBackend};
use azure::azure_hl::{CoreGraphicsAcceleratedBackend, Direct2DBackend, SkiaBackend};

/// Global flags for Servo, usually set on the command line.
pub struct Opts {
    /// The initial URLs to load.
    urls: ~[~str],

    /// The rendering backend to use (`-r`).
    render_backend: BackendType,

    /// How many threads to use for CPU rendering (`-t`).
    ///
    /// NB: This is not currently used. All rendering is sequential.
    n_render_threads: uint,

    /// True to use GPU rendering, false to use CPU rendering (`-g`). Note that compositing is
    /// always done on the GPU.
    gpu_rendering: bool,

    /// The maximum size of each tile, in pixels (`-s`).
    tile_size: uint,

    /// `None` to disable the profiler, `Some` and an interval in seconds to enable the profiler
    /// and produce output on that interval (`-p`).
    profiler_period: Option<f64>,

    /// A scale factor to apply to tiles, to allow rendering tiles at higher resolutions for
    /// testing pan and zoom code (`-z`).
    zoom: uint,
}

#[allow(non_implicitly_copyable_typarams)]
pub fn from_cmdline_args(args: &[~str]) -> Opts {
    use std::getopts;

    let args = args.tail();

    let opts = ~[
        getopts::optflag(~"g"), // GPU rendering
        getopts::optopt(~"o"),  // output file
        getopts::optopt(~"r"),  // rendering backend
        getopts::optopt(~"s"),  // size of tiles
        getopts::optopt(~"t"),  // threads to render with
        getopts::optflagopt(~"p"),  // profiler flag and output interval
        getopts::optopt(~"z"),  // zoom level
    ];

    let opt_match = match getopts::getopts(args, opts) {
      result::Ok(m) => { copy m }
      result::Err(f) => { fail!(getopts::fail_str(copy f)) }
    };
    let urls = if opt_match.free.is_empty() {
        fail!(~"servo asks that you provide 1 or more URLs")
    } else {
        copy opt_match.free
    };

    let render_backend = match getopts::opt_maybe_str(&opt_match, ~"r") {
        Some(backend_str) => {
            if backend_str == ~"direct2d" {
                Direct2DBackend
            } else if backend_str == ~"core-graphics" {
                CoreGraphicsBackend
            } else if backend_str == ~"core-graphics-accelerated" {
                CoreGraphicsAcceleratedBackend
            } else if backend_str == ~"cairo" {
                CairoBackend
            } else if backend_str == ~"skia" {
                SkiaBackend
            } else {
                fail!(~"unknown backend type")
            }
        }
        None => SkiaBackend
    };

    let tile_size: uint = match getopts::opt_maybe_str(&opt_match, ~"s") {
        Some(tile_size_str) => uint::from_str(tile_size_str).get(),
        None => 512,
    };

    let n_render_threads: uint = match getopts::opt_maybe_str(&opt_match, ~"t") {
        Some(n_render_threads_str) => uint::from_str(n_render_threads_str).get(),
        None => 1,      // FIXME: Number of cores.
    };

    let profiler_period: Option<f64> =
        // if only flag is present, default to 5 second period
        match getopts::opt_default(&opt_match, ~"p", ~"5") {
        Some(period) => Some(f64::from_str(period).get()),
        None => None,
    };

    let zoom: uint = match getopts::opt_maybe_str(&opt_match, ~"z") {
        Some(zoom_str) => uint::from_str(zoom_str).get(),
        None => 1,
    };

    let gpu_rendering = getopts::opt_present(&opt_match, ~"g");

    Opts {
        urls: urls,
        render_backend: render_backend,
        n_render_threads: n_render_threads,
        gpu_rendering: gpu_rendering,
        tile_size: tile_size,
        profiler_period: profiler_period,
        zoom: zoom,
    }
}
