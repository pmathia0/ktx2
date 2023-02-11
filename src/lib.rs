pub mod texture;
pub mod vk_format;

#[cfg(test)]
mod tests {
    use half::f16;

    use crate::{texture::TextureKtx2, vk_format::VkFormat};

    #[test]
    fn it_works() {
        let mut tex: TextureKtx2 = TextureKtx2::new(2, 2, VkFormat::R16_SFLOAT);
        for i in 0..tex.pixelHeight {
            for j in 0..tex.pixelWidth {
                tex.write_f16(j, i, f16::from_f32(1500f32));
            }
        }
        tex.write_to_ktx2("output.ktx2").unwrap();

        assert_eq!(2 + 2, 4);
    }
}
