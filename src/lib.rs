pub mod vk_format;
pub(crate) mod header;
pub(crate) mod index;
pub(crate) mod level;
pub(crate) mod dfd;
pub mod pixel;

pub mod texture;

pub mod filter;
#[cfg(test)]
mod tests {
    use half::f16;

    use crate::{texture::TextureKtx2, vk_format::VkFormat, pixel::Pixel};

    #[test]
    fn test_r16_sfloat() {
        let size = 2u32;

        let mut tex: TextureKtx2 = TextureKtx2::new(size, size, VkFormat::R16_SFLOAT);
        for i in 0..size {
            for j in 0..size {
                tex.write_pixel(j, i, Pixel::R16_SFLOAT(f16::from_f32(1500f32)));
            }
        }
        tex.write_to_ktx2("output_r16_sfloat.ktx2").unwrap();

        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn test_rgba8_uint() {
        let size = 2u32;

        let mut tex: TextureKtx2 = TextureKtx2::new(size, size, VkFormat::R8G8B8A8_UNORM);
        tex.write_pixel(0, 0, Pixel::R8G8B8A8_UNORM([255,0,0,255]));
        tex.write_pixel(1, 0, Pixel::R8G8B8A8_UNORM([0,255,0,255]));
        tex.write_pixel(0, 1, Pixel::R8G8B8A8_UNORM([0,0,255,255]));
        tex.write_pixel(1, 1, Pixel::R8G8B8A8_UNORM([255,255,0,255]));

        tex.write_to_ktx2("output_rgba8_unorm.ktx2").unwrap();

        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn test_bc1_new() {
        let size = 4u32;

        let mut tex: TextureKtx2 = TextureKtx2::new(size, size, VkFormat::BC1_RGB_UNORM_BLOCK);
        
        tex.write_to_ktx2("output_bc1_rgba_unorm.ktx2").unwrap();

        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn test_bc1_load() {
        let mut tex: TextureKtx2 = TextureKtx2::read_from_ktx2("output_bc1_rgba_unorm.ktx2").unwrap();
        
        tex.write_to_ktx2("output_bc1_rgba_unorm2.ktx2").unwrap();

        assert_eq!(2 + 2, 4);
    }
}
