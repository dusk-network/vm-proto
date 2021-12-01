mod ext {
    #[link(wasm_import_module = "env")]
    extern "C" {
        pub fn debug(ofs: &u8, len: i32);
    }
}

pub fn debug(string: &'static str) {
    let bytes = string.as_bytes();
    unsafe { ext::debug(&bytes[0], bytes.len() as i32) }
}

/// Store backend over FFI
#[cfg(not(feature = "host"))]
mod ffi_store;

/// Store backend over FFI
#[cfg(not(feature = "host"))]
pub use ffi_store::*;

#[cfg(not(feature = "host"))]
pub mod helpers;
#[cfg(not(feature = "host"))]
pub use helpers::*;
