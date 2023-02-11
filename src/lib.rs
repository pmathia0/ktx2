pub mod vk_format;
pub(crate) mod header;
pub(crate) mod index;
pub(crate) mod level;
pub(crate) mod dfd;

pub mod texture;

pub mod filter;
#[cfg(test)]
mod tests {
    use half::f16;

    use crate::{texture::TextureKtx2, vk_format::VkFormat};

    #[test]
    fn it_works() {
        let size = 2u32;
        let mut tex: TextureKtx2 = TextureKtx2::new(size, size, VkFormat::R16_SFLOAT);
        for i in 0..size {
            for j in 0..size {
                tex.write_f16(j, i, f16::from_f32(1500f32));
            }
        }
        tex.write_to_ktx2("output.ktx2").unwrap();

        assert_eq!(2 + 2, 4);
    }
}
