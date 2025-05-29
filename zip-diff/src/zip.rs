use crate::cdh::CentralDirectoryHeader;
use crate::compress::compress;
use crate::dd::{DataDescriptor, U32or64};
use crate::eocd::{
    EndOfCentralDirectoryRecord, Zip64EndOfCentralDirectoryLocator,
    Zip64EndOfCentralDirectoryRecord,
};
use crate::fields::{CompressionMethod, GeneralPurposeFlag};
use crate::lfh::LocalFileHeader;
use crate::utils::{binwrite_option, BinCount};
use anyhow::{Context, Result};
use binwrite::BinWrite;
use educe::Educe;
use std::fmt::{self, Formatter};

#[derive(BinWrite, Clone, Default, Debug)]
pub struct ZipArchive {
    pub files: Vec<FileEntry>,
    pub cd: Vec<CentralDirectoryHeader>,
    #[binwrite(with(binwrite_option))]
    pub zip64_eocdr: Option<Zip64EndOfCentralDirectoryRecord>,
    #[binwrite(with(binwrite_option))]
    pub zip64_eocdl: Option<Zip64EndOfCentralDirectoryLocator>,
    pub eocdr: EndOfCentralDirectoryRecord,
}

#[derive(BinWrite, Clone, Default, Educe)]
#[educe(Debug)]
pub struct FileEntry {
    pub lfh: LocalFileHeader,
    #[educe(Debug(method = fmt_len))]
    pub data: Vec<u8>,
    #[binwrite(with(binwrite_option))]
    pub dd: Option<DataDescriptor>,
}

fn fmt_len<T>(v: &[T], f: &mut Formatter<'_>) -> fmt::Result {
    write!(f, "Vec<{}> ({})", std::any::type_name::<T>(), v.len())
}

impl FileEntry {
    pub fn new(
        name: &str,
        uncompressed_data: &[u8],
        compression_method: CompressionMethod,
        force_zip64: bool,
        use_dd: bool,
    ) -> Result<Self> {
        let compressed_data = compress(compression_method, uncompressed_data)?;
        let crc32 = crc32fast::hash(uncompressed_data);

        let mut lfh = LocalFileHeader {
            compression_method,
            file_name_length: name.len().try_into().context("File name too long")?,
            file_name: name.into(),
            ..Default::default()
        };

        // When data descriptor is used, also set these fields for CDH.
        lfh.crc32 = crc32;
        lfh.set_compressed_size(compressed_data.len(), force_zip64);
        lfh.set_uncompressed_size(uncompressed_data.len(), force_zip64);

        let dd = if use_dd {
            lfh.general_purpose_flag
                .insert(GeneralPurposeFlag::DataDescriptor);

            let compressed_size = if let Some(size) = lfh.zip64.compressed_size {
                lfh.keep_empty_zip64 = true;
                U32or64::U64(size)
            } else {
                U32or64::U32(lfh.compressed_size)
            };

            let uncompressed_size = if let Some(size) = lfh.zip64.original_size {
                lfh.keep_empty_zip64 = true;
                U32or64::U64(size)
            } else {
                U32or64::U32(lfh.uncompressed_size)
            };

            Some(DataDescriptor {
                signature: Some(DataDescriptor::SIGNATURE),
                crc32,
                compressed_size,
                uncompressed_size,
            })
        } else {
            None
        };

        Ok(Self {
            lfh,
            data: compressed_data,
            dd,
        })
    }

    pub fn push_into_cd(
        &self,
        cd: &mut Vec<CentralDirectoryHeader>,
        offset: &mut usize,
    ) -> Result<()> {
        let mut cdh: CentralDirectoryHeader = self.into();
        cdh.set_offset(*offset, false);
        cdh.finalize()?;
        cd.push(cdh);
        *offset += self.byte_count()?;
        Ok(())
    }
}

impl ZipArchive {
    pub fn add_file(
        &mut self,
        name: &str,
        uncompressed_data: &[u8],
        compression_method: CompressionMethod,
        force_zip64: bool,
        use_dd: bool,
    ) -> Result<()> {
        self.files.push(FileEntry::new(
            name,
            uncompressed_data,
            compression_method,
            force_zip64,
            use_dd,
        )?);
        Ok(())
    }

    pub fn add_simple(&mut self, name: &str, data: &[u8]) -> Result<()> {
        self.add_file(name, data, CompressionMethod::STORED, false, false)
    }

    pub fn set_eocd(&mut self, force_zip64: bool) -> Result<()> {
        let mut offset = 0;
        if let Some(last_cdh) = self.cd.last() {
            offset += last_cdh.relative_header_offset as usize;
        }
        if let Some(last_file) = self.files.last() {
            offset += last_file.byte_count()?;
        }

        let mut zip64_eocdr = Zip64EndOfCentralDirectoryRecord {
            this_disk_cdh_count: self.cd.len() as u64,
            total_cdh_count: self.cd.len() as u64,
            size_of_cd: self.cd.byte_count()? as u64,
            offset_of_cd_wrt_starting_disk: offset as u64,
            ..Default::default()
        };

        if let (false, Ok(eocdr)) = (
            force_zip64,
            TryInto::<EndOfCentralDirectoryRecord>::try_into(&zip64_eocdr),
        ) {
            self.eocdr = eocdr;
            self.zip64_eocdl = None;
            self.zip64_eocdr = None;
        } else {
            zip64_eocdr.finalize()?;
            self.eocdr = EndOfCentralDirectoryRecord::all_ff();
            self.zip64_eocdl = Some(Zip64EndOfCentralDirectoryLocator {
                signature: Zip64EndOfCentralDirectoryLocator::SIGNATURE,
                zip64_eocdr_disk_number: 0,
                zip64_eocdr_offset: offset as u64 + zip64_eocdr.size_of_cd,
                total_number_of_disks: 1,
            });
            self.zip64_eocdr = Some(zip64_eocdr);
        }

        Ok(())
    }

    pub fn finalize(&mut self) -> Result<()> {
        self.cd.clear();

        let mut offset: usize = 0;

        for file in &mut self.files {
            let mut cdh: CentralDirectoryHeader = (&*file).into();
            cdh.set_offset(offset, false);
            cdh.finalize()?;
            self.cd.push(cdh);
            file.lfh.finalize()?;
            offset += file.byte_count().context("Failed to count file bytes")?;
        }

        self.set_eocd(false)
    }

    pub fn set_offsets(&mut self, base: usize) -> Result<()> {
        let mut offset: usize = base;

        for (file, cdh) in self.files.iter_mut().zip(self.cd.iter_mut()) {
            cdh.set_offset(offset, false);
            cdh.finalize()?;
            file.lfh.finalize()?;
            offset += file.byte_count().context("Failed to count file bytes")?;
        }

        self.set_eocd(false)
    }
}
