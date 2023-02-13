use crate::vk_format::{VkFormat, get_format_pixel_size_bytes};

#[derive(Clone)]
#[repr(C)]
pub struct DFDSampleType {
    pub row_0: u32,
    pub row_1: u32,
    pub row_2: u32,
    pub row_3: u32,
}

#[derive(Clone)]
#[repr(C)]
pub struct BasicDataFormatDescriptor {
    pub dfd_total_size: u32,
    pub row_0: u32,
    pub row_1: u32,
    pub row_2: u32,
    pub row_3: u32,
    pub row_4: u32,
    pub row_5: u32,
    pub samples: Vec<DFDSampleType>
}

impl BasicDataFormatDescriptor {
    pub fn new(vk_format: VkFormat) -> Self {
        match vk_format {
            VkFormat::R16_SFLOAT => {
                let samples = vec![
                    DFDSampleType {
                        row_0: 0 << 0 | 15 << 16 | 0b11000000 << 24,
                        row_1: 0u32,
                        row_2: 0xBF800000u32, // IEEE 754 floating-point representation for -1.0f
                        row_3: 0x3F800000u32, // IEEE 754 floating-point representation for 1.0f
                    }];
                let descriptor_block_size = (24 + std::mem::size_of::<DFDSampleType>() * samples.len()) as u32;
                BasicDataFormatDescriptor {
                    dfd_total_size: descriptor_block_size + 4,
                    row_0: 0u32,
                    row_1: 2 << 0  | descriptor_block_size << 16,
                    row_2: 1 << 0 | 1 << 8 | 1 << 16 | 0 << 24,
                    row_3: 0u32,
                    row_4: get_format_pixel_size_bytes(vk_format) as u32,
                    row_5: 0u32,
                    samples
                }
            },
            VkFormat::R8G8B8A8_UNORM => {
                let samples = vec![
                    // R
                    DFDSampleType {
                        row_0: 0 << 0 | 7 << 16 | 0b0000_0000 << 24,
                        row_1: 0u32,
                        row_2: 0,
                        row_3: 255
                    },
                    // G
                    DFDSampleType {
                        row_0: 8 << 0 | 7 << 16 | 0b0000_0001 << 24,
                        row_1: 0u32,
                        row_2: 0,
                        row_3: 255
                    },
                    // B
                    DFDSampleType {
                        row_0: 16 << 0 | 7 << 16 | 0b0000_0010 << 24,
                        row_1: 0u32,
                        row_2: 0,
                        row_3: 255
                    },
                    // A
                    DFDSampleType {
                        row_0: 24 << 0 | 7 << 16 | 0b0000_1111 << 24,
                        row_1: 0u32,
                        row_2: 0,
                        row_3: 255
                    }
                ];
                let descriptor_block_size = (24 + std::mem::size_of::<DFDSampleType>() * samples.len()) as u32;
                BasicDataFormatDescriptor {
                    dfd_total_size: descriptor_block_size + 4,
                    row_0: 0u32,
                    row_1: 2 << 0  | descriptor_block_size << 16,
                    row_2: 1 << 0 | 1 << 8 | 1 << 16 | 0 << 24,
                    row_3: 0u32,
                    row_4: get_format_pixel_size_bytes(vk_format) as u32,
                    row_5: 0u32,
                    samples
                }
            }
            _ => panic!("Unsupported format {:?}", vk_format)
        }
    }
}

impl Default for BasicDataFormatDescriptor {
    fn default() -> Self {
        Self {
            dfd_total_size: 24u32,
            row_0: 0u32,
            row_1: 0u32,
            row_2: 0u32,
            row_3: 0u32,
            row_4: 0u32,
            row_5: 0u32,
            samples: Vec::new()
        }
    }
}