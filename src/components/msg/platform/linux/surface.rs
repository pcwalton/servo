//! X11-specific implementation of cross-process surfaces. This uses X pixmaps.

use platform::surface::NativeSurfaceAzureMethods;

use azure::AzSkiaGrGLSharedSurfaceRef;
use layers::platform::surface::NativeSurface;
use std::cast;

impl NativeSurfaceAzureMethods for NativeSurface {
    fn from_azure_surface(surface: AzSkiaGrGLSharedSurfaceRef) -> NativeSurface {
        unsafe {
            NativeSurface::from_pixmap(cast::transmute(surface))
        }
    }
}

