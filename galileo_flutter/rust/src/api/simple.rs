use crate::core::{IS_INITIALIZED};
use log::debug;



#[flutter_rust_bridge::frb(init)]
pub fn init_app() {
    crate::core::init_logger();
}


pub fn galileo_flutter_init(ffi_ptr: i64) {
    let mut is_initialized = IS_INITIALIZED.lock().unwrap();
    if *is_initialized {
        return;
    }
    irondash_dart_ffi::irondash_init_ffi(ffi_ptr as *mut std::ffi::c_void);

    debug!("Done initializing galileo flutter");
    *is_initialized = true;
}