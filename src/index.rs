use crate::level::Level;

#[repr(C)]
pub(crate) struct Index {
    pub dfdByteOffset: u32,
    pub dfdByteLength: u32,
    pub kvdByteOffset: u32,
    pub kvdByteLength: u32,
    pub sgdByteOffset: u64,
    pub sgdByteLength: u64,
    pub levels: Vec<Level>
}