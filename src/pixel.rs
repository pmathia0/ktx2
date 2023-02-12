use half::f16;

#[allow(non_camel_case_types)]
pub enum Pixel {
    R16_SFLOAT(f16),
    R8G8B8A8_UINT([u8; 4])
}