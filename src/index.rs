use crate::level::Level;

#[repr(C)]
pub(crate) struct Index {
    pub dfd_byte_offset: u32,
    pub dfd_byte_length: u32,
    pub kvd_byte_offset: u32,
    pub kvd_byte_length: u32,
    pub sgd_byte_offset: u64,
    pub sgd_byte_length: u64,
    pub levels: Vec<Level>
}