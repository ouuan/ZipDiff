use crate::extra::ExtraFieldType;
use crate::fields::CompressionMethod;
use crate::utils::{binwrite_option, binwrite_transform, BinCount};
use anyhow::{Context, Result};
use binwrite::BinWrite;
use educe::Educe;

#[derive(BinWrite, Clone, Educe)]
#[educe(Debug, Default)]
pub struct EndOfCentralDirectoryRecord {
    #[educe(Default = Self::SIGNATURE)]
    #[educe(Debug(method(std::fmt::LowerHex::fmt)))]
    pub signature: u32,
    pub number_of_this_disk: u16,
    /// number of the disk with the start of the central directory
    pub start_of_cd_disk_number: u16,
    /// total number of entries in the central directory on this disk
    pub this_disk_cdh_count: u16,
    /// total number of entries in the central directory
    pub total_cdh_count: u16,
    /// size of the central directory
    pub size_of_cd: u32,
    /// offset of start of central directory with respect to the starting disk number
    pub offset_of_cd_wrt_starting_disk: u32,
    pub zip_file_comment_length: u16,
    pub zip_file_comment: Vec<u8>,
}

impl EndOfCentralDirectoryRecord {
    pub const SIGNATURE: u32 = 0x06054b50;

    pub fn all_ff() -> Self {
        Self {
            number_of_this_disk: u16::MAX,
            start_of_cd_disk_number: u16::MAX,
            this_disk_cdh_count: u16::MAX,
            total_cdh_count: u16::MAX,
            size_of_cd: u32::MAX,
            offset_of_cd_wrt_starting_disk: u32::MAX,
            ..Default::default()
        }
    }
}

impl TryFrom<&Zip64EndOfCentralDirectoryRecord> for EndOfCentralDirectoryRecord {
    type Error = anyhow::Error;

    fn try_from(zip64: &Zip64EndOfCentralDirectoryRecord) -> Result<Self, Self::Error> {
        Ok(Self {
            number_of_this_disk: zip64.number_of_this_disk.try_into()?,
            start_of_cd_disk_number: zip64.start_of_cd_disk_number.try_into()?,
            this_disk_cdh_count: zip64.this_disk_cdh_count.try_into()?,
            total_cdh_count: zip64.total_cdh_count.try_into()?,
            size_of_cd: zip64.size_of_cd.try_into()?,
            offset_of_cd_wrt_starting_disk: zip64.offset_of_cd_wrt_starting_disk.try_into()?,
            ..Default::default()
        })
    }
}

#[derive(BinWrite, Clone, Educe)]
#[educe(Debug, Default)]
pub struct Zip64EndOfCentralDirectoryLocator {
    #[educe(Debug(method(std::fmt::LowerHex::fmt)))]
    #[educe(Default = Self::SIGNATURE)]
    pub signature: u32,
    /// number of the disk with the start of the zip64 end of central directory
    pub zip64_eocdr_disk_number: u32,
    /// relative offset of the zip64 end of central directory record
    pub zip64_eocdr_offset: u64,
    #[educe(Default = 1)]
    pub total_number_of_disks: u32,
}

impl Zip64EndOfCentralDirectoryLocator {
    pub const SIGNATURE: u32 = 0x07064b50;

    pub fn from_offset(offset: u64) -> Self {
        Self {
            zip64_eocdr_offset: offset,
            ..Default::default()
        }
    }
}

#[derive(BinWrite, Clone, Educe)]
#[educe(Debug, Default)]
pub struct Zip64EndOfCentralDirectoryRecord {
    #[educe(Default = Self::SIGNATURE)]
    #[educe(Debug(method(std::fmt::LowerHex::fmt)))]
    pub signature: u32,
    pub size: u64,
    #[educe(Default = 20)]
    pub version_made_by: u16,
    #[educe(Default = 20)]
    pub version_needed: u16,
    pub number_of_this_disk: u32,
    /// number of the disk with the start of the central directory
    pub start_of_cd_disk_number: u32,
    /// total number of entries in the central directory on this disk
    pub this_disk_cdh_count: u64,
    /// total number of entries in the central directory
    pub total_cdh_count: u64,
    /// size of the central directory
    pub size_of_cd: u64,
    /// offset of start of central directory with respect to the starting disk number
    pub offset_of_cd_wrt_starting_disk: u64,
    #[binwrite(with(binwrite_option))]
    pub v2: Option<Zip64EocdrV2>,
    pub extensible_data_sector: Vec<Zip64ExtensibleDataSector>,
}

#[derive(BinWrite, Clone, Debug, Default)]
pub struct Zip64EocdrV2 {
    #[binwrite(with(binwrite_transform))]
    pub compression_method: CompressionMethod,
    pub compressed_size: u64,
    pub original_size: u64,
    pub encrypt_alg: u16,
    pub key_bit_len: u16,
    pub encrypt_flags: u16,
    pub hash_alg: u16,
    pub hash_len: u16,
    pub hash_data: Vec<u8>,
}

impl Zip64EndOfCentralDirectoryRecord {
    pub const SIGNATURE: u32 = 0x06064b50;

    pub fn finalize(&mut self) -> Result<()> {
        for field in &mut self.extensible_data_sector {
            field.finalize()?;
        }
        self.size =
            self.extensible_data_sector
                .byte_count()
                .context("Failed to count ZIP64 EOCDR extensible data sector")? as u64
                + 44;
        if let Some(v2) = &self.v2 {
            self.size += v2.byte_count()? as u64;
        }
        Ok(())
    }

    pub fn use_v2(&mut self) -> Result<()> {
        self.version_made_by = 62;
        self.version_needed = 62;
        self.v2 = Some(Zip64EocdrV2 {
            compressed_size: self.size_of_cd,
            original_size: self.size_of_cd,
            ..Default::default()
        });
        self.finalize()
    }
}

#[derive(BinWrite, Clone, Debug)]
pub struct Zip64ExtensibleDataSector {
    pub header_id: u16,
    pub size: u32,
    #[binwrite(with(binwrite_transform))]
    pub data: Box<dyn ExtraFieldType>,
}

impl Zip64ExtensibleDataSector {
    pub fn finalize(&mut self) -> Result<()> {
        self.header_id = self.data.header_id();
        self.size = self
            .data
            .binary_encode()
            .context("Failed to count extensible data sector size")?
            .len()
            .try_into()
            .context("Extensible data sector too long")?;
        Ok(())
    }
}

impl<T: ExtraFieldType> From<T> for Zip64ExtensibleDataSector {
    fn from(data: T) -> Self {
        Self {
            header_id: 0,
            size: 0,
            data: Box::new(data),
        }
    }
}
