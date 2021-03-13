use obs_sys as sys;

/// Saves the obs graphics context, runs the provided function, then restores the original graphics
/// context
pub fn isolate_context<Ret, F>(f: F) -> Ret where
    F: FnOnce() -> Ret,
{
    let previous_ctx = unsafe {
        let ctx = sys::gs_get_context();
        sys::gs_leave_context();
        ctx
    };
    let ret = f();
    unsafe {
        sys::gs_enter_context(previous_ctx);
    }
    ret
}

unsafe fn enter_graphics() {
    trace!("obs_enter_graphics");
    sys::obs_enter_graphics();
}

unsafe fn leave_graphics() {
    trace!("obs_leave_graphics");
    sys::obs_leave_graphics();
}

/// Enters the obs graphics context, runs the provided function, then leaves the obs graphics
/// context. see: `obs_enter_graphics` and `obs_leave_graphics` in the obs-studio API
pub fn with_graphics<Ret, F: FnOnce() -> Ret>(f: F) -> Ret {
    unsafe { enter_graphics(); }
    let ret = f();
    unsafe { leave_graphics(); }
    ret
}
