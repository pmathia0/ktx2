pub mod texture;
pub mod vk_format;

#[cfg(test)]
mod tests {
    use crate::{texture::TextureKtx2, vk_format::VkFormat};

    #[test]
    fn it_works() {
        let mut tex: TextureKtx2 = TextureKtx2::new(16, 16, VkFormat::R8G8B8A8_UNORM);
        for i in 0..tex.pixelHeight {
            for j in 0..tex.pixelWidth {
                tex.write_pixel(j, i, &[255u8,255u8,1u8,255u8]);
            }
        }
        tex.write_to_ktx2("output.ktx2").unwrap();

        assert_eq!(2 + 2, 4);
    }
}
