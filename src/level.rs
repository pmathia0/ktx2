#[derive(Copy, Clone)]
#[repr(C)]
pub struct Level {
    pub byte_offset: u64,
    pub byte_length: u64,
    pub uncompressed_byte_length: u64
}