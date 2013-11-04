/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use compositing::*;

use geom::size::Size2D;
use servo_msg::compositor_msg::{ChangeReadyState, ChangeRenderState, DeleteLayer, Exit};
use servo_msg::compositor_msg::{InvalidateRect, NewLayer, Paint, SetIds, SetLayerClipRect};
use servo_msg::compositor_msg::{SetLayerPageSize};
use servo_msg::constellation_msg::{FrameTreeReceivedMsg, InitLoadUrlMsg, ResizedWindowMsg};
use servo_msg::constellation_msg;
use servo_util::url;

/// Starts the compositor, which listens for messages on the specified port.
///
/// This is the null compositor which doesn't draw anything to the screen.
/// It's intended for headless testing.
pub fn run_compositor(compositor: &mut CompositorTask) {
    // Send over the initial URL(s).
    for url in compositor.opts.urls.iter() {
        compositor.comm.send(InitLoadUrlMsg(url::make_url(url.clone(), None).to_str()));
    }

    loop {
        let msg = compositor.comm.recv();
        match msg {
            Exit => break,

            SetIds(*) => {
                compositor.comm.send(ResizedWindowMsg(Size2D(400u, 300u)));
                compositor.comm.send(FrameTreeReceivedMsg);
            }

            // Explicitly list ignored messages so that when we add a new one,
            // we'll notice and think about whether it needs a response, like
            // SetIds.

            NewLayer(*) | SetLayerPageSize(*) | SetLayerClipRect(*) | DeleteLayer(*) |
            Paint(*) | InvalidateRect(*) | ChangeReadyState(*) | ChangeRenderState(*)
                => ()
        }
    }

    compositor.comm.send(constellation_msg::ExitMsg);
}
