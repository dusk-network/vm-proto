use rkyv::Archive;

#[repr(C)]
pub struct Scratch(u32);

impl Scratch {
    pub fn write<T: Archive>(_t: &T) -> Self {
        loop {}
    }
}
