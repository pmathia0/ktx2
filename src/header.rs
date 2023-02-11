use crate::vk_format::VkFormat;

#[derive(Clone, Copy)]
#[repr(C)]
pub(crate) struct Header {
    pub identifier: [u8; 12],
    pub vkFormat: VkFormat,
    pub typeSize: u32,
    pub pixelWidth: u32,
    pub pixelHeight: u32,
    pub pixelDepth: u32,
    pub layerCount: u32,
    pub faceCount: u32,
    pub levelCount: u32,
    pub supercompressionScheme: u32,
}