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
    pub row_0: u32,
    pub row_1: u32,
    pub row_2: u32,
    pub row_3: u32,
    pub row_4: u32,
    pub row_5: u32,
    pub samples: Vec<DFDSampleType>
}