use crate::lfh::LocalFileHeader;
use crate::utils::binwrite_option;
use binwrite::{BinWrite, WriterOption};
use std::io::{Result, Write};

#[derive(BinWrite, Clone, Default, Debug)]
pub struct DataDescriptor {
    #[binwrite(with(binwrite_option))]
    pub signature: Option<u32>,
    pub crc32: u32,
    #[binwrite(with(binwrite_u32or64))]
    pub compressed_size: U32or64,
    #[binwrite(with(binwrite_u32or64))]
    pub uncompressed_size: U32or64,
}

#[derive(Clone, Debug)]
pub enum U32or64 {
    U32(u32),
    U64(u64),
}

impl DataDescriptor {
    pub const SIGNATURE: u32 = 0x08074b50;
}

fn binwrite_u32or64<W: Write>(val: &U32or64, writer: &mut W, options: &WriterOption) -> Result<()> {
    match val {
        U32or64::U32(val) => val.write_options(writer, options),
        U32or64::U64(val) => val.write_options(writer, options),
    }
}

impl U32or64 {
    pub fn saturate(&self) -> u32 {
        match self {
            U32or64::U32(val) => *val,
            U32or64::U64(val) => {
                if *val > u32::MAX as u64 {
                    u32::MAX
                } else {
                    *val as u32
                }
            }
        }
    }
}

impl Default for U32or64 {
    fn default() -> Self {
        Self::U32(0)
    }
}

impl From<&LocalFileHeader> for DataDescriptor {
    fn from(value: &LocalFileHeader) -> Self {
        Self {
            signature: Some(Self::SIGNATURE),
            crc32: value.crc32,
            compressed_size: match value.zip64.compressed_size {
                None => U32or64::U32(value.compressed_size),
                Some(size) => U32or64::U64(size),
            },
            uncompressed_size: match value.zip64.original_size {
                None => U32or64::U32(value.uncompressed_size),
                Some(size) => U32or64::U64(size),
            },
        }
    }
}
