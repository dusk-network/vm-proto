mod ext {
    extern "C" {
        pub fn debug(ofs: *const u8, len: usize);
    }
}

pub fn debug(string: &'static str) {
    let bytes = string.as_bytes();
    unsafe { ext::debug(&bytes[0], bytes.len()) }
}
