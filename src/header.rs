use crate::vk_format::VkFormat;

#[derive(Clone, Copy)]
#[repr(C)]
pub(crate) struct Header {
    pub identifier: [u8; 12],
    pub vk_format: VkFormat,
    pub type_size: u32,
    pub pixel_width: u32,
    pub pixel_height: u32,
    pub pixel_depth: u32,
    pub layer_count: u32,
    pub face_count: u32,
    pub level_count: u32,
    pub supercompression_scheme: u32,
}