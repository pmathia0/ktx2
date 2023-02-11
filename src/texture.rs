#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

extern crate byteorder;
extern crate anyhow;

use byteorder::{LittleEndian, WriteBytesExt, ReadBytesExt};
use std::fs::File;
use std::io;
use std::io::Write;
use std::io::Read;
use std::io::Cursor;
use half::f16;
use std::f32;

use crate::dfd::DFDSampleType;
use crate::vk_format::*;
use crate::header::Header;
use crate::index::Index;
use crate::level::Level;
use crate::dfd::BasicDataFormatDescriptor;

use crate::filter::*;

#[repr(C, align(1))]
pub struct TextureKtx2 {
    header: Header,

    // Index 
    index: Index,

    // Data Format Descriptor 
    pub dfdTotalSize: u32,
    pub dfDescriptorBlock: Vec<BasicDataFormatDescriptor>,

    // Key/Value Data 
    pub keyAndValueData: [u8; 52],

    // Supercompression Global Data 
    pub supercompressionGlobalData: Vec<u8>,

    // Mip Level Array 
    pub levelImages: Vec<u8>
}

impl TextureKtx2 {

    pub fn new(width: u32, height: u32, format: VkFormat) -> Self {
        let typeSize = get_format_type_size_bytes(format);

        let mut texture = TextureKtx2 {
            header: Header {
                identifier: [
                    0xAB, 0x4B, 0x54, 0x58, 0x20, 0x32, 0x30, 0xBB, 0x0D, 0x0A, 0x1A, 0x0A
                ],
                vkFormat: format,
                typeSize,
                pixelWidth: width,
                pixelHeight: height,
                pixelDepth: 0,
                layerCount: 0,
                faceCount: 1,
                levelCount: 1,
                supercompressionScheme: 0,
            },
            index: Index {
                // Index 
                dfdByteOffset: 104u32,
                dfdByteLength: 44u32,
                kvdByteOffset: 148u32,
                kvdByteLength: 52u32,
                sgdByteOffset: 0u64,
                sgdByteLength: 0u64,
                // Level Index 
                levels: Vec::new(),
            },

            // Data Format Descriptor 
            dfdTotalSize: 44u32,
            dfDescriptorBlock: Vec::new(),

            // Key/Value Data 
            keyAndValueData: [
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
            supercompressionGlobalData: Vec::new(),

            // Mip Level Array 
            levelImages: Vec::new()
        };
        texture.index.levels.resize(1, Level {
            byteOffset: 200u64,
            byteLength: (width*height*typeSize) as u64,
            uncompressedByteLength: (width*height*typeSize) as u64
        });
        texture.dfDescriptorBlock.resize(1, BasicDataFormatDescriptor {
            row_0: 0u32,
            row_1: 0u32,
            row_2: 0u32,
            row_3: 0u32,
            row_4: 0u32,
            row_5: 0u32,
            samples: Vec::new()
        });
        texture.dfDescriptorBlock[0].row_0 = 0u32;
        texture.dfDescriptorBlock[0].row_1 = 2 << 0  | 40 << 16;
        texture.dfDescriptorBlock[0].row_2 = 1 << 0 | 1 << 8 | 1 << 16 | 0 << 24;
        texture.dfDescriptorBlock[0].row_3 = 0u32;
        texture.dfDescriptorBlock[0].row_4 = 2u32;
        texture.dfDescriptorBlock[0].row_5 = 0u32;
        texture.dfDescriptorBlock[0].samples.resize(1, DFDSampleType {
            row_0: 0u32,
            row_1: 0u32,
            row_2: 0xBF800000u32, // IEEE 754 floating-point representation for -1.0f
            row_3: 0x3F800000u32, // IEEE 754 floating-point representation for 1.0f
        });
        texture.dfDescriptorBlock[0].samples[0].row_0 = 0 << 0 | 15 << 16 | 0b11000000 << 24;
        
        texture.supercompressionGlobalData.resize(0, 0x00);
        texture.levelImages.resize(texture.index.levels[0].byteLength as usize, 0x00);
        texture
    }

    pub fn read_f16(&self, x: u32, y: u32) -> f16 {
        let index: usize = (x as usize) * 2 as usize + (self.header.pixelWidth as usize) * (y as usize) * 2 as usize;
        let mut a: [u8; 2] = [0,0];
        a[0] = self.levelImages[index];
        a[1] = self.levelImages[index+1];
        let value = half::f16::from_le_bytes(a);
        value
    }

    pub fn write_f16(&mut self, x: u32, y: u32, value: f16) {
        let index: usize = (x as usize) * 2usize + (self.header.pixelWidth as usize) * (y as usize) * 2usize;
        let mut data = vec![];
        data.write_u16::<LittleEndian>(f16::to_bits(value)).unwrap();
        self.levelImages[index] = data[0];
        self.levelImages[index + 1] = data[1];
    }

    pub fn write_pixel(&mut self, x: u32, y: u32, pixel: &[u8; 4]) {
        let index: usize = (x as usize) * 4usize + (self.header.pixelWidth as usize) * (y as usize) * 4usize;
        self.levelImages[index] = pixel[0];
        self.levelImages[index + 1] = pixel[1];
        self.levelImages[index + 2] = pixel[2];
        self.levelImages[index + 3] = pixel[3];
    }

    pub fn write_to_ktx2(&mut self, file_name: &str) -> io::Result<()> {
        let mut buffer = File::create(file_name)?;
        buffer.write_all(&self.header.identifier)?;

        let mut header = vec![];
        header.write_u32::<LittleEndian>(self.header.vkFormat as u32).unwrap();
        header.write_u32::<LittleEndian>(self.header.typeSize).unwrap();
        header.write_u32::<LittleEndian>(self.header.pixelWidth).unwrap();
        header.write_u32::<LittleEndian>(self.header.pixelHeight).unwrap();
        header.write_u32::<LittleEndian>(self.header.pixelDepth).unwrap();
        header.write_u32::<LittleEndian>(self.header.layerCount).unwrap();
        header.write_u32::<LittleEndian>(self.header.faceCount).unwrap();
        header.write_u32::<LittleEndian>(self.header.levelCount).unwrap();
        header.write_u32::<LittleEndian>(self.header.supercompressionScheme).unwrap();
        buffer.write_all(&header)?;
        
        let mut index = vec![];
        index.write_u32::<LittleEndian>(self.index.dfdByteOffset).unwrap();
        index.write_u32::<LittleEndian>(self.index.dfdByteLength).unwrap();
        index.write_u32::<LittleEndian>(self.index.kvdByteOffset).unwrap();
        index.write_u32::<LittleEndian>(self.index.kvdByteLength).unwrap();
        index.write_u64::<LittleEndian>(self.index.sgdByteOffset).unwrap();
        index.write_u64::<LittleEndian>(self.index.sgdByteLength).unwrap();
        buffer.write_all(&index)?;

        let mut levels = vec![];
        for level in &self.index.levels {
            levels.write_u64::<LittleEndian>(level.byteOffset).unwrap();
            levels.write_u64::<LittleEndian>(level.byteLength).unwrap();
            levels.write_u64::<LittleEndian>(level.uncompressedByteLength).unwrap();
        }
        buffer.write_all(&levels)?;

        let mut dfd_size = vec![];
        dfd_size.write_u32::<LittleEndian>(self.dfdTotalSize).unwrap();
        buffer.write_all(&dfd_size)?;
        for descriptor in &self.dfDescriptorBlock {
            let mut dfd = vec![];
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
        buffer.write_all(&self.keyAndValueData)?;

        buffer.write_all(&self.supercompressionGlobalData)?;
        buffer.write_all(&self.levelImages)?;
        Ok(())
    }
    
    pub fn read_from_ktx2(file_name: &str) -> Result<TextureKtx2, anyhow::Error> {
        
        let mut file = File::open(file_name)?;

        let mut buffer: Vec<u8> = vec![];
        file.read_to_end(&mut buffer)?;
        
        let mut vkFormat_rdr = Cursor::new(&buffer[12..16]);
        let vkFormat: VkFormat = unsafe { std::mem::transmute(vkFormat_rdr.read_u32::<LittleEndian>().unwrap()) };
        // println!("vkFormat: {:?}", vkFormat);
        let mut typeSize_rdr = Cursor::new(&buffer[16..20]);
        let typeSize: u32 = typeSize_rdr.read_u32::<LittleEndian>()?;
        // println!("typeSize: {:?}", typeSize);
        let mut pixelWidth_rdr = Cursor::new(&buffer[20..24]);
        let pixelWidth: u32 = pixelWidth_rdr.read_u32::<LittleEndian>()?;
        // println!("pixelWidth: {:?}", pixelWidth);
        let mut pixelHeight_rdr = Cursor::new(&buffer[24..28]);
        let pixelHeight: u32 = pixelHeight_rdr.read_u32::<LittleEndian>()?;
        // println!("pixelHeight: {:?}", pixelHeight);
        let mut pixelDepth_rdr = Cursor::new(&buffer[28..32]);
        let pixelDepth: u32 = pixelDepth_rdr.read_u32::<LittleEndian>()?;
        // println!("pixelDepth: {:?}", pixelDepth);
        let mut layerCount_rdr = Cursor::new(&buffer[32..36]);
        let layerCount: u32 = layerCount_rdr.read_u32::<LittleEndian>()?;
        // println!("layerCount: {:?}", layerCount);
        let mut faceCount_rdr = Cursor::new(&buffer[36..40]);
        let faceCount: u32 = faceCount_rdr.read_u32::<LittleEndian>()?;
        // println!("faceCount: {:?}", faceCount);
        let mut levelCount_rdr = Cursor::new(&buffer[40..44]);
        let levelCount: u32 = levelCount_rdr.read_u32::<LittleEndian>()?;
        // println!("levelCount: {:?}", levelCount);
        let mut supercompressionScheme_rdr = Cursor::new(&buffer[44..48]);
        let supercompressionScheme: u32 = supercompressionScheme_rdr.read_u32::<LittleEndian>()?;
        // println!("supercompressionScheme: {:?}", supercompressionScheme);
        let mut dfdByteOffset_rdr = Cursor::new(&buffer[48..52]);
        let dfdByteOffset: u32 = dfdByteOffset_rdr.read_u32::<LittleEndian>()?;
        // println!("dfdByteOffset: {:?}", dfdByteOffset);
        let mut dfdByteLength_rdr = Cursor::new(&buffer[52..56]);
        let dfdByteLength: u32 = dfdByteLength_rdr.read_u32::<LittleEndian>()?;
        // println!("dfdByteLength: {:?}", dfdByteLength);
        let mut kvdByteOffset_rdr = Cursor::new(&buffer[56..60]);
        let kvdByteOffset: u32 = kvdByteOffset_rdr.read_u32::<LittleEndian>()?;
        // println!("kvdByteOffset: {:?}", kvdByteOffset);
        let mut kvdByteLength_rdr = Cursor::new(&buffer[60..64]);
        let kvdByteLength: u32 = kvdByteLength_rdr.read_u32::<LittleEndian>()?;
        // println!("kvdByteLength: {:?}", kvdByteLength);
        let mut sgdByteOffset_rdr = Cursor::new(&buffer[64..72]);
        let sgdByteOffset: u64 = sgdByteOffset_rdr.read_u64::<LittleEndian>()?;
        // println!("sgdByteOffset: {:?}", sgdByteOffset);
        let mut sgdByteLength_rdr = Cursor::new(&buffer[72..80]);
        let sgdByteLength: u64 = sgdByteLength_rdr.read_u64::<LittleEndian>()?;
        // println!("sgdByteLength: {:?}", sgdByteLength);

        // read level info
        let mut levels: Vec<Level> = vec![];
        for l in 0..levelCount {
            // println!("Level {:?}", l);
            let mut byteOffset_rdr = Cursor::new(&buffer[80+l as usize*8..88+l as usize*8]);
            let mut byteLength_rdr = Cursor::new(&buffer[88+l as usize *8..96+l as usize *8]);
            let mut uncompressedByteLength_rdr = Cursor::new(&buffer[96+l as usize *8..104+l as usize *8]);

            let byteOffset: u64 = byteOffset_rdr.read_u64::<LittleEndian>()?;
            let byteLength: u64 = byteLength_rdr.read_u64::<LittleEndian>()?;
            let uncompressedByteLength: u64 = uncompressedByteLength_rdr.read_u64::<LittleEndian>()?;
            
            // println!("\tbyteOffset: {:?}", byteOffset);
            // println!("\tbyteLength: {:?}", byteLength);
            // println!("\tuncompressedByteLength: {:?}", uncompressedByteLength);

            levels.push(Level {
                byteOffset: byteOffset,
                byteLength: byteLength,
                uncompressedByteLength: uncompressedByteLength 
            });
        }

        // read DFD (assuming only 1)
        let mut dfdTotalSize_rdr = Cursor::new(&buffer[dfdByteOffset as usize..dfdByteOffset as usize +4]);
        let dfdTotalSize: u32 = dfdTotalSize_rdr.read_u32::<LittleEndian>()?;
        // println!("dfdByteLength ?= dfdTotalSize : {:?}", dfdByteLength == dfdTotalSize);
        let mut dfd_rdr0 = Cursor::new(&buffer[dfdByteOffset as usize +4..dfdByteOffset as usize +8]);
        let mut dfd_rdr1 = Cursor::new(&buffer[dfdByteOffset as usize +8..dfdByteOffset as usize +12]);
        let mut dfd_rdr2 = Cursor::new(&buffer[dfdByteOffset as usize +12..dfdByteOffset as usize +16]);
        let mut dfd_rdr3 = Cursor::new(&buffer[dfdByteOffset as usize +16..dfdByteOffset as usize +20]);
        let mut dfd_rdr4 = Cursor::new(&buffer[dfdByteOffset as usize +20..dfdByteOffset as usize +24]);
        let mut dfd_rdr5 = Cursor::new(&buffer[dfdByteOffset as usize +24..dfdByteOffset as usize +28]);
        let mut dfd = BasicDataFormatDescriptor {
            row_0: dfd_rdr0.read_u32::<LittleEndian>().unwrap(),
            row_1: dfd_rdr1.read_u32::<LittleEndian>().unwrap(),
            row_2: dfd_rdr2.read_u32::<LittleEndian>().unwrap(),
            row_3: dfd_rdr3.read_u32::<LittleEndian>().unwrap(),
            row_4: dfd_rdr4.read_u32::<LittleEndian>().unwrap(),
            row_5: dfd_rdr5.read_u32::<LittleEndian>().unwrap(),
            samples: vec![]
        };
        // read DFDSampleType (assuming 1)
        let mut DFDSampleType_rdr0 = Cursor::new(&buffer[dfdByteOffset as usize +28..dfdByteOffset as usize +32]);
        let mut DFDSampleType_rdr1 = Cursor::new(&buffer[dfdByteOffset as usize +32..dfdByteOffset as usize +36]);
        let mut DFDSampleType_rdr2 = Cursor::new(&buffer[dfdByteOffset as usize +36..dfdByteOffset as usize +40]);
        let mut DFDSampleType_rdr3 = Cursor::new(&buffer[dfdByteOffset as usize +40..dfdByteOffset as usize +44]);
        let sample = DFDSampleType {
            row_0: DFDSampleType_rdr0.read_u32::<LittleEndian>().unwrap(),
            row_1: DFDSampleType_rdr1.read_u32::<LittleEndian>().unwrap(),
            row_2: DFDSampleType_rdr2.read_u32::<LittleEndian>().unwrap(),
            row_3: DFDSampleType_rdr3.read_u32::<LittleEndian>().unwrap()
        };
        dfd.samples.push(sample);
        Ok(TextureKtx2 {
            header: Header {
                identifier: [
                    0xAB, 0x4B, 0x54, 0x58, 0x20, 0x32, 0x30, 0xBB, 0x0D, 0x0A, 0x1A, 0x0A
                ],
                vkFormat: vkFormat,
                typeSize: typeSize,
                pixelWidth: pixelWidth,
                pixelHeight: pixelHeight,
                pixelDepth: pixelDepth,
                layerCount: layerCount,
                faceCount: faceCount,
                levelCount: levelCount,
                supercompressionScheme: supercompressionScheme,
            },
            
            dfdTotalSize: dfdTotalSize,
            dfDescriptorBlock: vec![dfd],
            keyAndValueData: [
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
            supercompressionGlobalData: vec![0u8; 0],
            levelImages: buffer[levels[0].byteOffset as usize..].to_vec(),
            index: Index {
                dfdByteOffset: dfdByteOffset,
                dfdByteLength: dfdByteLength,
                kvdByteOffset: kvdByteOffset,
                kvdByteLength: kvdByteLength,
                sgdByteOffset: sgdByteOffset,
                sgdByteLength: sgdByteLength,
                levels: levels
            },            
        })
    }

    pub fn vertical_sample(image: &mut TextureKtx2, new_height: u32, filter: &mut Filter) -> TextureKtx2 {
        let width = image.header.pixelWidth;
        let height = image.header.pixelHeight;
        let mut out: TextureKtx2 = TextureKtx2::new(width, new_height, image.header.vkFormat);
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
                    let p = half::f16::to_f32(image.read_f16(x, left + i as u32));
                    let k1 = p;
                    t += k1 * w;
                }
    
                let t1 = t / sum;
                let t = half::f16::from_f32(clamp(t1, 0.0, max));
    
                out.write_f16(x, outy, t);
            }
        }
        out
    }

    pub fn horizontal_sample(image: &mut TextureKtx2, new_width: u32, filter: &mut Filter) -> TextureKtx2 {
        let width = image.header.pixelWidth;
        let height = image.header.pixelHeight;
        let mut out: TextureKtx2 = TextureKtx2::new(new_width, height, image.header.vkFormat);
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
                    let p = half::f16::to_f32(image.read_f16(left + i as u32, y));
                    let k1 = p;
                    t += k1 * w;
                }
    
                let t1 = t / sum;
                let t = half::f16::from_f32(clamp(t1, 0.0, max));
    
                out.write_f16(outx, y, t);
            }
        }
        out
    }

    pub fn resize(image: &mut TextureKtx2, nwidth: u32, nheight: u32, filter: FilterType) -> TextureKtx2 {
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
        let out = TextureKtx2::horizontal_sample(&mut tmp, nwidth, &mut method);
        out
    }
}