use openvr_sys as sys;

pub trait ErrorType: Eq + Sized {
    fn non_error() -> Self;

    fn is_error(&self) -> bool {
        *self != Self::non_error()
    }
}

impl ErrorType for sys::EVRInitError {
    #[inline(always)]
    fn non_error() -> Self {
        sys::EVRInitError::EVRInitError_VRInitError_None
    }
}

impl ErrorType for sys::EVRCompositorError {
    #[inline(always)]
    fn non_error() -> Self {
        sys::EVRCompositorError::EVRCompositorError_VRCompositorError_None
    }
}

impl ErrorType for sys::EVROverlayError {
    #[inline(always)]
    fn non_error() -> Self {
        sys::EVROverlayError::EVROverlayError_VROverlayError_None
    }
}

pub trait ErrorTypeExt: Sized {
    fn into_result(self) -> Result<Self, Self>;

    fn into_empty_result(self) -> Result<(), Self> {
        self.into_result().map(|_| ())
    }
}

impl<T> ErrorTypeExt for T where
    T: ErrorType + Sized,
{
    fn into_result(self) -> Result<Self, Self> {
        if self.is_error() {
            Err(self)
        } else {
            Ok(self)
        }
    }
}
