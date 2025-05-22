extern crate anyhow;
extern crate byteorder;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use half::f16;
use std::f32;
use std::fs::File;
use std::io;
use std::io::Cursor;
use std::io::Read;
use std::io::Write;
use std::mem::size_of_val;

use crate::dfd::BasicDataFormatDescriptor;
use crate::dfd::DFDSampleType;
use crate::header::Header;
use crate::index::Index;
use crate::level::Level;
use crate::pixel::Pixel;
use crate::vk_format::*;

use crate::filter::*;

#[repr(C, align(1))]
#[derive(Clone)]
pub struct TextureKtx2 {
    pub header: Header,

    // Index
    index: Index,

    // Data Format Descriptor
    pub dfd_descriptor_block: Vec<BasicDataFormatDescriptor>,

    // Key/Value Data
    pub key_value_data: [u8; 52],

    // Supercompression Global Data
    pub supercompression_global_data: Vec<u8>,

    // Mip Level Array
    pub level_images: Vec<u8>,
}

impl TextureKtx2 {
    pub fn new(width: u32, height: u32, format: VkFormat) -> Self {
        let type_size = get_format_type_size_bytes(format);

        let header = Header {
            identifier: [
                0xAB, 0x4B, 0x54, 0x58, 0x20, 0x32, 0x30, 0xBB, 0x0D, 0x0A, 0x1A, 0x0A,
            ],
            vk_format: format,
            type_size,
            pixel_width: width,
            pixel_height: height,
            pixel_depth: 0,
            layer_count: 0,
            face_count: 1,
            level_count: 1,
            supercompression_scheme: 0,
        };

        let dfd = BasicDataFormatDescriptor::new(format);

        let mut levels = Vec::new();
        let pixel_size = get_format_pixel_size_bytes(format);
        let byte_length = (width as f32 * height as f32 * pixel_size) as u64;

        levels.push(Level {
            byte_offset: 200u64,
            byte_length,
            uncompressed_byte_length: byte_length,
        });

        let index_size = (32 + std::mem::size_of::<Level>() * levels.len()) as u32;
        let mut index = Index {
            // Index
            dfd_byte_offset: 0,
            dfd_byte_length: dfd.dfd_total_size,
            kvd_byte_offset: 148u32,
            kvd_byte_length: 52u32,
            sgd_byte_offset: 0u64,
            sgd_byte_length: 0u64,
            // Level Index
            levels,
        };
        let dfd_byte_offset = size_of_val(&header) as u32 + index_size;
        index.dfd_byte_offset = dfd_byte_offset;
        index.kvd_byte_offset = dfd_byte_offset + dfd.dfd_total_size;
        index.sgd_byte_offset = 0; //index.kvd_byte_offset as u64 + 52;
        index.levels[0].byte_offset = index.kvd_byte_offset as u64 + 52;

        TextureKtx2 {
            header,
            index,

            // Data Format Descriptor
            dfd_descriptor_block: vec![dfd],

            // Key/Value Data
            key_value_data: [
                0x12, 0x00, 0x00, 0x00, // 18 bytes for first entry
                0x4B, 0x54, 0x58, 0x6F, // KTXo
                0x72, 0x69, 0x65, 0x6E, // rien
                0x74, 0x61, 0x74, 0x69, // tati
                0x6F, 0x6E, 0x00, 0x72, // on NUL r
                0x64, 0x00, 0x00, 0x00, // d  <3 bytes of valuePadding>
                0x18, 0x00, 0x00, 0x00, // 24 bytes for second entry
                0x4B, 0x54, 0x58, 0x77, // KTXw
                0x72, 0x69, 0x74, 0x65, // rite
                0x72, 0x00, 0x53, 0x65, // r NULL Se
                0x6e, 0x69, 0x6f, 0x72, // nior
                0x53, 0x4b, 0x59, 0x5f, // SKY_
                0x64, 0x74, 0x32, 0x00, // dt2 NULL
            ],

            // Supercompression Global Data
            supercompression_global_data: Vec::new(),

            // Mip Level Array
            level_images: vec![0x00; byte_length as usize],
        }
    }

    pub fn read_pixel(&self, x: u32, y: u32) -> Pixel {
        let pixel_size = get_format_pixel_size_bytes(self.header.vk_format);
        let index: usize = ((x as f32) * pixel_size
            + (self.header.pixel_width as f32) * (y as f32) * pixel_size)
            as usize;
        match self.header.vk_format {
            VkFormat::R16_SFLOAT => {
                let mut a: [u8; 2] = [0, 0];
                a[0] = self.level_images[index];
                a[1] = self.level_images[index + 1];
                let value = half::f16::from_le_bytes(a);
                Pixel::R16_SFLOAT(value)
            }
            _ => panic!(
                "Unsupported format for direct pixel write {:?}",
                self.header.vk_format
            ),
        }
    }

    pub fn write_pixel(&mut self, x: u32, y: u32, pixel: Pixel) {
        // TODO check format and Pixel format
        let pixel_size = get_format_pixel_size_bytes(self.header.vk_format);
        let index: usize = ((x as f32) * pixel_size
            + (self.header.pixel_width as f32) * (y as f32) * pixel_size)
            as usize;
        let mut data = vec![];
        match pixel {
            Pixel::R16_SFLOAT(value) => {
                data.write_u16::<LittleEndian>(f16::to_bits(value)).unwrap();
                self.level_images[index] = data[0];
                self.level_images[index + 1] = data[1];
            }
            Pixel::R8G8B8A8_UNORM(data) => {
                self.level_images[index] = data[0];
                self.level_images[index + 1] = data[1];
                self.level_images[index + 2] = data[2];
                self.level_images[index + 3] = data[3];
            }
            _ => panic!(
                "Unsupported format for direct pixel write {:?}",
                self.header.vk_format
            ),
        }
    }

    pub fn write_to_ktx2(&mut self, file_name: &str) -> io::Result<()> {
        let mut buffer = File::create(file_name)?;
        buffer.write_all(&self.header.identifier)?;

        let mut header = vec![];
        header
            .write_u32::<LittleEndian>(self.header.vk_format as u32)
            .unwrap();
        header
            .write_u32::<LittleEndian>(self.header.type_size)
            .unwrap();
        header
            .write_u32::<LittleEndian>(self.header.pixel_width)
            .unwrap();
        header
            .write_u32::<LittleEndian>(self.header.pixel_height)
            .unwrap();
        header
            .write_u32::<LittleEndian>(self.header.pixel_depth)
            .unwrap();
        header
            .write_u32::<LittleEndian>(self.header.layer_count)
            .unwrap();
        header
            .write_u32::<LittleEndian>(self.header.face_count)
            .unwrap();
        header
            .write_u32::<LittleEndian>(self.header.level_count)
            .unwrap();
        header
            .write_u32::<LittleEndian>(self.header.supercompression_scheme)
            .unwrap();
        buffer.write_all(&header)?;

        let mut index = vec![];
        index
            .write_u32::<LittleEndian>(self.index.dfd_byte_offset)
            .unwrap();
        index
            .write_u32::<LittleEndian>(self.index.dfd_byte_length)
            .unwrap();
        index
            .write_u32::<LittleEndian>(self.index.kvd_byte_offset)
            .unwrap();
        index
            .write_u32::<LittleEndian>(self.index.kvd_byte_length)
            .unwrap();
        index
            .write_u64::<LittleEndian>(self.index.sgd_byte_offset)
            .unwrap();
        index
            .write_u64::<LittleEndian>(self.index.sgd_byte_length)
            .unwrap();
        buffer.write_all(&index)?;

        let mut levels = vec![];
        for level in &self.index.levels {
            levels.write_u64::<LittleEndian>(level.byte_offset).unwrap();
            levels.write_u64::<LittleEndian>(level.byte_length).unwrap();
            levels
                .write_u64::<LittleEndian>(level.uncompressed_byte_length)
                .unwrap();
        }
        buffer.write_all(&levels)?;

        for descriptor in &self.dfd_descriptor_block {
            let mut dfd = vec![];
            dfd.write_u32::<LittleEndian>(descriptor.dfd_total_size)
                .unwrap();
            dfd.write_u32::<LittleEndian>(descriptor.row_0).unwrap();
            dfd.write_u32::<LittleEndian>(descriptor.row_1).unwrap();
            dfd.write_u32::<LittleEndian>(descriptor.row_2).unwrap();
            dfd.write_u32::<LittleEndian>(descriptor.row_3).unwrap();
            dfd.write_u32::<LittleEndian>(descriptor.row_4).unwrap();
            dfd.write_u32::<LittleEndian>(descriptor.row_5).unwrap();
            buffer.write_all(&dfd)?;
            for sample in &descriptor.samples {
                let mut spl = vec![];
                spl.write_u32::<LittleEndian>(sample.row_0).unwrap();
                spl.write_u32::<LittleEndian>(sample.row_1).unwrap();
                spl.write_u32::<LittleEndian>(sample.row_2).unwrap();
                spl.write_u32::<LittleEndian>(sample.row_3).unwrap();
                buffer.write_all(&spl)?;
            }
        }
        buffer.write_all(&self.key_value_data)?;

        buffer.write_all(&self.supercompression_global_data)?;
        buffer.write_all(&self.level_images)?;
        Ok(())
    }

    pub fn read_from_ktx2(file_name: &str) -> Result<TextureKtx2, anyhow::Error> {
        let mut file = File::open(file_name)?;

        let mut buffer: Vec<u8> = vec![];
        file.read_to_end(&mut buffer)?;

        let mut vk_format_rdr = Cursor::new(&buffer[12..16]);
        let vk_format: VkFormat =
            unsafe { std::mem::transmute(vk_format_rdr.read_u32::<LittleEndian>().unwrap()) };
        // println!("vkFormat: {:?}", vkFormat);
        let mut type_size_rdr = Cursor::new(&buffer[16..20]);
        let type_size: u32 = type_size_rdr.read_u32::<LittleEndian>()?;
        // println!("typeSize: {:?}", typeSize);
        let mut pixel_width_rdr = Cursor::new(&buffer[20..24]);
        let pixel_width: u32 = pixel_width_rdr.read_u32::<LittleEndian>()?;
        // println!("pixelWidth: {:?}", pixelWidth);
        let mut pixel_height_rdr = Cursor::new(&buffer[24..28]);
        let pixel_height: u32 = pixel_height_rdr.read_u32::<LittleEndian>()?;
        // println!("pixelHeight: {:?}", pixelHeight);
        let mut pixel_depth_rdr = Cursor::new(&buffer[28..32]);
        let pixel_depth: u32 = pixel_depth_rdr.read_u32::<LittleEndian>()?;
        // println!("pixelDepth: {:?}", pixelDepth);
        let mut layer_count_rdr = Cursor::new(&buffer[32..36]);
        let layer_count: u32 = layer_count_rdr.read_u32::<LittleEndian>()?;
        // println!("layerCount: {:?}", layerCount);
        let mut face_count_rdr = Cursor::new(&buffer[36..40]);
        let face_count: u32 = face_count_rdr.read_u32::<LittleEndian>()?;
        // println!("faceCount: {:?}", faceCount);
        let mut level_count_rdr = Cursor::new(&buffer[40..44]);
        let level_count: u32 = level_count_rdr.read_u32::<LittleEndian>()?;
        // println!("levelCount: {:?}", levelCount);
        let mut supercompression_scheme_rdr = Cursor::new(&buffer[44..48]);
        let supercompression_scheme: u32 =
            supercompression_scheme_rdr.read_u32::<LittleEndian>()?;
        // println!("supercompressionScheme: {:?}", supercompressionScheme);
        let mut dfd_byte_offset_rdr = Cursor::new(&buffer[48..52]);
        let dfd_byte_offset: u32 = dfd_byte_offset_rdr.read_u32::<LittleEndian>()?;
        // println!("dfdByteOffset: {:?}", dfdByteOffset);
        let mut dfd_byte_length_rdr = Cursor::new(&buffer[52..56]);
        let dfd_byte_length: u32 = dfd_byte_length_rdr.read_u32::<LittleEndian>()?;
        // println!("dfdByteLength: {:?}", dfdByteLength);
        let mut kvd_byte_offset_rdr = Cursor::new(&buffer[56..60]);
        let kvd_byte_offset: u32 = kvd_byte_offset_rdr.read_u32::<LittleEndian>()?;
        // println!("kvdByteOffset: {:?}", kvdByteOffset);
        let mut kvd_byte_length_rdr = Cursor::new(&buffer[60..64]);
        let kvd_byte_length: u32 = kvd_byte_length_rdr.read_u32::<LittleEndian>()?;
        // println!("kvdByteLength: {:?}", kvdByteLength);
        let mut sgd_byte_offset_rdr = Cursor::new(&buffer[64..72]);
        let sgd_byte_offset: u64 = sgd_byte_offset_rdr.read_u64::<LittleEndian>()?;
        // println!("sgdByteOffset: {:?}", sgdByteOffset);
        let mut sgd_byte_length_rdr = Cursor::new(&buffer[72..80]);
        let sgd_byte_length: u64 = sgd_byte_length_rdr.read_u64::<LittleEndian>()?;
        // println!("sgdByteLength: {:?}", sgdByteLength);

        // read level info
        let mut levels: Vec<Level> = vec![];
        for l in 0..level_count {
            // println!("Level {:?}", l);
            let mut byte_offset_rdr =
                Cursor::new(&buffer[80 + l as usize * 8..88 + l as usize * 8]);
            let mut byte_length_rdr =
                Cursor::new(&buffer[88 + l as usize * 8..96 + l as usize * 8]);
            let mut uncompressed_byte_length_rdr =
                Cursor::new(&buffer[96 + l as usize * 8..104 + l as usize * 8]);

            let byte_offset: u64 = byte_offset_rdr.read_u64::<LittleEndian>()?;
            let byte_length: u64 = byte_length_rdr.read_u64::<LittleEndian>()?;
            let uncompressed_byte_length: u64 =
                uncompressed_byte_length_rdr.read_u64::<LittleEndian>()?;

            // println!("\tbyte_offset: {:?}", byte_offset);
            // println!("\tbyte_length: {:?}", _byte_length);
            // println!("\tuncompressed_byte_length: {:?}", uncompressed_byte_length);

            levels.push(Level {
                byte_offset,
                byte_length,
                uncompressed_byte_length,
            });
        }

        // read DFD (assuming only 1)
        let mut dfd_total_size_rdr =
            Cursor::new(&buffer[dfd_byte_offset as usize..dfd_byte_offset as usize + 4]);
        let mut dfd_rdr0 =
            Cursor::new(&buffer[dfd_byte_offset as usize + 4..dfd_byte_offset as usize + 8]);
        let mut dfd_rdr1 =
            Cursor::new(&buffer[dfd_byte_offset as usize + 8..dfd_byte_offset as usize + 12]);
        let mut dfd_rdr2 =
            Cursor::new(&buffer[dfd_byte_offset as usize + 12..dfd_byte_offset as usize + 16]);
        let mut dfd_rdr3 =
            Cursor::new(&buffer[dfd_byte_offset as usize + 16..dfd_byte_offset as usize + 20]);
        let mut dfd_rdr4 =
            Cursor::new(&buffer[dfd_byte_offset as usize + 20..dfd_byte_offset as usize + 24]);
        let mut dfd_rdr5 =
            Cursor::new(&buffer[dfd_byte_offset as usize + 24..dfd_byte_offset as usize + 28]);
        let mut dfd = BasicDataFormatDescriptor {
            dfd_total_size: dfd_total_size_rdr.read_u32::<LittleEndian>().unwrap(),
            row_0: dfd_rdr0.read_u32::<LittleEndian>().unwrap(),
            row_1: dfd_rdr1.read_u32::<LittleEndian>().unwrap(),
            row_2: dfd_rdr2.read_u32::<LittleEndian>().unwrap(),
            row_3: dfd_rdr3.read_u32::<LittleEndian>().unwrap(),
            row_4: dfd_rdr4.read_u32::<LittleEndian>().unwrap(),
            row_5: dfd_rdr5.read_u32::<LittleEndian>().unwrap(),
            samples: vec![],
        };
        // read DFDSampleType (assuming 1)
        let mut dfd_sample_type_rdr0 =
            Cursor::new(&buffer[dfd_byte_offset as usize + 28..dfd_byte_offset as usize + 32]);
        let mut dfd_sample_type_rdr1 =
            Cursor::new(&buffer[dfd_byte_offset as usize + 32..dfd_byte_offset as usize + 36]);
        let mut dfd_sample_type_rdr2 =
            Cursor::new(&buffer[dfd_byte_offset as usize + 36..dfd_byte_offset as usize + 40]);
        let mut dfd_sample_type_rdr3 =
            Cursor::new(&buffer[dfd_byte_offset as usize + 40..dfd_byte_offset as usize + 44]);
        let sample = DFDSampleType {
            row_0: dfd_sample_type_rdr0.read_u32::<LittleEndian>().unwrap(),
            row_1: dfd_sample_type_rdr1.read_u32::<LittleEndian>().unwrap(),
            row_2: dfd_sample_type_rdr2.read_u32::<LittleEndian>().unwrap(),
            row_3: dfd_sample_type_rdr3.read_u32::<LittleEndian>().unwrap(),
        };
        dfd.samples.push(sample);
        Ok(TextureKtx2 {
            header: Header {
                identifier: [
                    0xAB, 0x4B, 0x54, 0x58, 0x20, 0x32, 0x30, 0xBB, 0x0D, 0x0A, 0x1A, 0x0A,
                ],
                vk_format,
                type_size,
                pixel_width,
                pixel_height,
                pixel_depth,
                layer_count,
                face_count,
                level_count,
                supercompression_scheme,
            },

            dfd_descriptor_block: vec![dfd],
            key_value_data: [
                0x12, 0x00, 0x00, 0x00, // 18 bytes for first entry
                0x4B, 0x54, 0x58, 0x6F, // KTXo
                0x72, 0x69, 0x65, 0x6E, // rien
                0x74, 0x61, 0x74, 0x69, // tati
                0x6F, 0x6E, 0x00, 0x72, // on NUL r
                0x64, 0x00, 0x00, 0x00, // d  <3 bytes of valuePadding>
                0x18, 0x00, 0x00, 0x00, // 24 bytes for second entry
                0x4B, 0x54, 0x58, 0x77, // KTXw
                0x72, 0x69, 0x74, 0x65, // rite
                0x72, 0x00, 0x53, 0x65, // r NULL Se
                0x6e, 0x69, 0x6f, 0x72, // nior
                0x53, 0x4b, 0x59, 0x5f, // SKY_
                0x64, 0x74, 0x32, 0x00, // dt2 NULL
            ],
            supercompression_global_data: vec![0u8; 0],
            level_images: buffer[levels[0].byte_offset as usize..].to_vec(),
            index: Index {
                dfd_byte_offset,
                dfd_byte_length,
                kvd_byte_offset,
                kvd_byte_length,
                sgd_byte_offset,
                sgd_byte_length,
                levels,
            },
        })
    }

    pub fn vertical_sample(
        image: &mut TextureKtx2,
        new_height: u32,
        filter: &mut Filter,
    ) -> TextureKtx2 {
        let width = image.header.pixel_width;
        let height = image.header.pixel_height;
        let mut out: TextureKtx2 = TextureKtx2::new(width, new_height, image.header.vk_format);
        let mut ws: Vec<f32> = Vec::new();

        let max: f32 = half::f16::to_f32(half::f16::MAX);
        let ratio = height as f32 / new_height as f32;
        let sratio = if ratio < 1.0 { 1.0 } else { ratio };
        let src_support = filter.support * sratio;

        for outy in 0..new_height {
            // For an explanation of this algorithm, see the comments
            // in horizontal_sample.
            let inputy = (outy as f32 + 0.5) * ratio;

            let left = (inputy - src_support).floor() as i64;
            let left = clamp(left, 0, <i64 as From<_>>::from(height) - 1) as u32;

            let right = (inputy + src_support).ceil() as i64;
            let right = clamp(
                right,
                <i64 as From<_>>::from(left) + 1,
                <i64 as From<_>>::from(height),
            ) as u32;

            let inputy = inputy - 0.5;

            ws.clear();
            let mut sum = 0.0;
            for i in left..right {
                let w = (filter.kernel)((i as f32 - inputy) / sratio);
                ws.push(w);
                sum += w;
            }

            for x in 0..width {
                let mut t = 0.0f32;

                for (i, w) in ws.iter().enumerate() {
                    let pixel = image.read_pixel(x, left + i as u32);
                    let p = match pixel {
                        Pixel::R16_SFLOAT(p) => half::f16::to_f32(p),
                        _ => panic!("Unsupported format"),
                    };
                    let k1 = p;
                    t += k1 * w;
                }

                let t1 = t / sum;
                let t = half::f16::from_f32(clamp(t1, 0.0, max));

                out.write_pixel(x, outy, Pixel::R16_SFLOAT(t));
            }
        }
        out
    }

    pub fn horizontal_sample(
        image: &mut TextureKtx2,
        new_width: u32,
        filter: &mut Filter,
    ) -> TextureKtx2 {
        let width = image.header.pixel_width;
        let height = image.header.pixel_height;
        let mut out: TextureKtx2 = TextureKtx2::new(new_width, height, image.header.vk_format);
        let mut ws: Vec<f32> = Vec::new();

        let max: f32 = half::f16::to_f32(half::f16::MAX);
        let ratio = width as f32 / new_width as f32;
        let sratio = if ratio < 1.0 { 1.0 } else { ratio };
        let src_support = filter.support * sratio;

        for outx in 0..new_width {
            // Find the point in the input image corresponding to the centre
            // of the current pixel in the output image.
            let inputx = (outx as f32 + 0.5) * ratio;

            // Left and right are slice bounds for the input pixels relevant
            // to the output pixel we are calculating.  Pixel x is relevant
            // if and only if (x >= left) && (x < right).

            // Invariant: 0 <= left < right <= width

            let left = (inputx - src_support).floor() as i64;
            let left = clamp(left, 0, <i64 as From<_>>::from(width) - 1) as u32;

            let right = (inputx + src_support).ceil() as i64;
            let right = clamp(
                right,
                <i64 as From<_>>::from(left) + 1,
                <i64 as From<_>>::from(width),
            ) as u32;

            // Go back to left boundary of pixel, to properly compare with i
            // below, as the kernel treats the centre of a pixel as 0.
            let inputx = inputx - 0.5;

            ws.clear();
            let mut sum = 0.0;
            for i in left..right {
                let w = (filter.kernel)((i as f32 - inputx) / sratio);
                ws.push(w);
                sum += w;
            }

            for y in 0..height {
                let mut t = 0.0;

                for (i, w) in ws.iter().enumerate() {
                    let pixel = image.read_pixel(left + i as u32, y);
                    let p = match pixel {
                        Pixel::R16_SFLOAT(p) => half::f16::to_f32(p),
                        _ => panic!("Unsupported format"),
                    };
                    let k1 = p;
                    t += k1 * w;
                }

                let t1 = t / sum;
                let t = half::f16::from_f32(clamp(t1, 0.0, max));

                out.write_pixel(outx, y, Pixel::R16_SFLOAT(t));
            }
        }
        out
    }

    pub fn resize(
        image: &mut TextureKtx2,
        nwidth: u32,
        nheight: u32,
        filter: FilterType,
    ) -> TextureKtx2 {
        let mut method = match filter {
            FilterType::Nearest => Filter {
                kernel: Box::new(box_kernel),
                support: 0.0,
            },
            FilterType::Lanczos3 => Filter {
                kernel: Box::new(lanczos3_kernel),
                support: 3.0,
            },
        };

        let mut tmp = TextureKtx2::vertical_sample(image, nheight, &mut method);
        TextureKtx2::horizontal_sample(&mut tmp, nwidth, &mut method)
    }
}
