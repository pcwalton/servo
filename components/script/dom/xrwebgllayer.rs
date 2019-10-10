/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use canvas_traits::webgl::WebGLFramebufferId;
use crate::dom::bindings::codegen::Bindings::XRViewBinding::{XREye, XRViewMethods};
use crate::dom::bindings::codegen::Bindings::XRWebGLLayerBinding;
use crate::dom::bindings::codegen::Bindings::XRWebGLLayerBinding::XRWebGLLayerInit;
use crate::dom::bindings::codegen::Bindings::XRWebGLLayerBinding::XRWebGLLayerMethods;
use crate::dom::bindings::codegen::Bindings::WebGLRenderingContextBinding::WebGLRenderingContextBinding::WebGLRenderingContextMethods;
use crate::dom::bindings::codegen::Bindings::WebGLRenderingContextBinding::WebGLRenderingContextConstants as constants;
use crate::dom::bindings::error::Error;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector, DomObject};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::globalscope::GlobalScope;
use crate::dom::webgl_validations::types::TexImageTarget;
use crate::dom::webglframebuffer::WebGLFramebuffer;
use crate::dom::webglrenderingcontext::WebGLRenderingContext;
use crate::dom::window::Window;
use crate::dom::xrsession::XRSession;
use crate::dom::xrview::XRView;
use crate::dom::xrviewport::XRViewport;
use dom_struct::dom_struct;
use js::rust::CustomAutoRooter;
use std::convert::TryInto;
use webxr_api::SwapChainId as WebXRSwapChainId;
use webxr_api::Views;

#[dom_struct]
pub struct XRWebGLLayer {
    reflector_: Reflector,
    antialias: bool,
    depth: bool,
    stencil: bool,
    alpha: bool,
    #[ignore_malloc_size_of = "ids don't malloc"]
    swap_chain_id: WebXRSwapChainId,
    context: Dom<WebGLRenderingContext>,
    session: Dom<XRSession>,
    framebuffer: Dom<WebGLFramebuffer>,
}

impl XRWebGLLayer {
    pub fn new_inherited(
        swap_chain_id: WebXRSwapChainId,
        session: &XRSession,
        context: &WebGLRenderingContext,
        init: &XRWebGLLayerInit,
        framebuffer: &WebGLFramebuffer,
    ) -> XRWebGLLayer {
        XRWebGLLayer {
            reflector_: Reflector::new(),
            antialias: init.antialias,
            depth: init.depth,
            stencil: init.stencil,
            alpha: init.alpha,
            swap_chain_id,
            context: Dom::from_ref(context),
            session: Dom::from_ref(session),
            framebuffer: Dom::from_ref(framebuffer),
        }
    }

    pub fn new(
        global: &GlobalScope,
        swap_chain_id: WebXRSwapChainId,
        session: &XRSession,
        context: &WebGLRenderingContext,
        init: &XRWebGLLayerInit,
        framebuffer: &WebGLFramebuffer,
    ) -> DomRoot<XRWebGLLayer> {
        reflect_dom_object(
            Box::new(XRWebGLLayer::new_inherited(
                swap_chain_id,
                session,
                context,
                init,
                framebuffer,
            )),
            global,
            XRWebGLLayerBinding::Wrap,
        )
    }

    /// https://immersive-web.github.io/webxr/#dom-xrwebgllayer-xrwebgllayer
    pub fn Constructor(
        global: &Window,
        session: &XRSession,
        context: &WebGLRenderingContext,
        init: &XRWebGLLayerInit,
    ) -> Fallible<DomRoot<Self>> {
        // Step 2
        if session.is_ended() {
            return Err(Error::InvalidState);
        }
        // XXXManishearth step 3: throw error if context is lost
        // XXXManishearth step 4: check XR compat flag for immersive sessions

        // Step 8.2. "Initialize layer’s framebuffer to a new opaque framebuffer created with context."
        let size = session.with_session(|session| session.recommended_framebuffer_resolution());
        let (swap_chain_id, framebuffer) = WebGLFramebuffer::maybe_new_webxr(session, context, size).ok_or(Error::Operation)?;

        // Step 8.3. "Allocate and initialize resources compatible with session’s XR device,
        // including GPU accessible memory buffers, as required to support the compositing of layer."

        // Step 8.4: "If layer’s resources were unable to be created for any reason,
        // throw an OperationError and abort these steps."

        // Step 9. "Return layer."
        Ok(XRWebGLLayer::new(
            &global.global(),
	    swap_chain_id,
            session,
            context,
            init,
            &framebuffer,
        ))
    }

    pub fn swap_chain_id(&self) -> WebXRSwapChainId {
        self.swap_chain_id
    }

    pub fn session(&self) -> &XRSession {
        &self.session
    }

    pub fn framebuffer(&self) -> &WebGLFramebuffer {
        &self.framebuffer
    }

    pub fn swap_buffers(&self) {
        if let WebGLFramebufferId::Opaque(id) = self.framebuffer.id() {
            self.context.swap_buffers(Some(id));
        }
    }
}

impl XRWebGLLayerMethods for XRWebGLLayer {
    /// https://immersive-web.github.io/webxr/#dom-xrwebgllayer-depth
    fn Depth(&self) -> bool {
        self.depth
    }

    /// https://immersive-web.github.io/webxr/#dom-xrwebgllayer-stencil
    fn Stencil(&self) -> bool {
        self.stencil
    }

    /// https://immersive-web.github.io/webxr/#dom-xrwebgllayer-antialias
    fn Antialias(&self) -> bool {
        self.antialias
    }

    /// https://immersive-web.github.io/webxr/#dom-xrwebgllayer-alpha
    fn Alpha(&self) -> bool {
        self.alpha
    }

    /// https://immersive-web.github.io/webxr/#dom-xrwebgllayer-context
    fn Context(&self) -> DomRoot<WebGLRenderingContext> {
        DomRoot::from_ref(&self.context)
    }

    /// https://immersive-web.github.io/webxr/#dom-xrwebgllayer-framebuffer
    fn Framebuffer(&self) -> DomRoot<WebGLFramebuffer> {
        DomRoot::from_ref(&self.framebuffer)
    }

    /// https://immersive-web.github.io/webxr/#dom-xrwebgllayer-framebufferwidth
    fn FramebufferWidth(&self) -> u32 {
        self.framebuffer
            .size()
            .unwrap_or((0, 0))
            .0
            .try_into()
            .unwrap_or(0)
    }

    /// https://immersive-web.github.io/webxr/#dom-xrwebgllayer-framebufferheight
    fn FramebufferHeight(&self) -> u32 {
        self.framebuffer
            .size()
            .unwrap_or((0, 0))
            .1
            .try_into()
            .unwrap_or(0)
    }

    /// https://immersive-web.github.io/webxr/#dom-xrwebgllayer-getviewport
    fn GetViewport(&self, view: &XRView) -> Option<DomRoot<XRViewport>> {
        if self.session != view.session() {
            return None;
        }

        let views = self.session.with_session(|s| s.views().clone());

        let viewport = match (view.Eye(), views) {
            (XREye::None, Views::Mono(view)) => view.viewport,
            (XREye::Left, Views::Stereo(view, _)) => view.viewport,
            (XREye::Right, Views::Stereo(_, view)) => view.viewport,
            // The spec doesn't really say what to do in this case
            // https://github.com/immersive-web/webxr/issues/769
            _ => return None,
        };

        Some(XRViewport::new(&self.global(), viewport))
    }
}
