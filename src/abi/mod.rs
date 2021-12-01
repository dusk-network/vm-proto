#[cfg(not(feature = "host"))]
mod ext {
    #[link(wasm_import_module = "env")]
    extern "C" {
        pub fn debug(ofs: &u8, len: i32);
    }
}

#[cfg(not(feature = "host"))]
pub fn debug(string: &'static str) {
    let bytes = string.as_bytes();
    unsafe { ext::debug(&bytes[0], bytes.len() as i32) }
}

// Host mockups of the ABI
#[cfg(feature = "host")]
pub fn debug(string: &'static str) {
    println!("HOST DEBUG: {}", string)
}

/// Store backend over FFI
#[cfg(not(feature = "host"))]
mod ffi_store;

/// Store backend over FFI
#[cfg(not(feature = "host"))]
pub use ffi_store::*;
