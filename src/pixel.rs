use half::f16;

#[allow(non_camel_case_types)]
pub enum Pixel {
    R16_SFLOAT(f16),
    R16G16B16A16_SFLOAT([f16; 4]),
    R8G8B8A8_UNORM([u8; 4]),
    BC1_RGB_UNORM_BLOCK,
}
