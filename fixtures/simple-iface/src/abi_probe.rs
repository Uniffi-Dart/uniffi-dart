//! Test-only ABI recorder. Never dereferences the supplied handle or status pointer.
use std::{ffi::c_void, sync::Mutex};
use uniffi::Handle;

static CAPTURE: Mutex<(u64, u32, usize)> = Mutex::new((0, 0, 0));

#[no_mangle]
pub extern "C" fn abi_capture(handle: Handle, marker: u32, status: *mut c_void) {
    *CAPTURE.lock().unwrap() = (handle.as_raw(), marker, status as usize);
}

#[no_mangle]
pub extern "C" fn abi_capture_clone(handle: Handle, status: *mut c_void) -> Handle {
    *CAPTURE.lock().unwrap() = (handle.as_raw(), 0, status as usize);
    handle
}

#[no_mangle]
pub extern "C" fn abi_captured_handle() -> u64 {
    CAPTURE.lock().unwrap().0
}
#[no_mangle]
pub extern "C" fn abi_captured_marker() -> u32 {
    CAPTURE.lock().unwrap().1
}
#[no_mangle]
pub extern "C" fn abi_captured_status() -> usize {
    CAPTURE.lock().unwrap().2
}
#[no_mangle]
pub extern "C" fn abi_return_handle() -> Handle {
    Handle::from_raw_unchecked(0x1234_5678_9abc_def0)
}
#[no_mangle]
pub extern "C" fn abi_invoke_callback(
    callback: extern "C" fn(Handle, u32, *mut c_void),
    handle: Handle,
    marker: u32,
    status: *mut c_void,
) {
    callback(handle, marker, status);
}
