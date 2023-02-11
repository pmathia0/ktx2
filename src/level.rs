#[derive(Copy, Clone)]
#[repr(C)]
pub struct Level {
    pub byteOffset: u64,
    pub byteLength: u64,
    pub uncompressedByteLength: u64
}