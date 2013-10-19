//! Declarations of types for cross-process surfaces.

use azure::AzSkiaGrGLSharedSurfaceRef;

pub trait NativeSurfaceAzureMethods {
    fn from_azure_surface(surface: AzSkiaGrGLSharedSurfaceRef) -> Self;
}

