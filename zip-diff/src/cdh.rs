use crate::extra::{ExtraField, Zip64ExtendedInfo};
use crate::fields::*;
use crate::lfh::LocalFileHeader;
use crate::utils::{binwrite_transform, BinCount};
use crate::zip::FileEntry;
use anyhow::{bail, Context, Result};
use binwrite::BinWrite;
use educe::Educe;

#[derive(BinWrite, Clone, Educe)]
#[educe(Debug, Default)]
pub struct CentralDirectoryHeader {
    #[educe(Default = Self::SIGNATURE)]
    #[educe(Debug(method(std::fmt::LowerHex::fmt)))]
    pub signature: u32,
    #[educe(Default = 20)]
    pub version_made_by: u16,
    #[educe(Default = 20)]
    pub version_needed: u16,
    #[binwrite(with(binwrite_transform))]
    pub general_purpose_flag: GeneralPurposeFlag,
    #[binwrite(with(binwrite_transform))]
    pub compression_method: CompressionMethod,
    pub last_mod: DosDateTime,
    #[educe(Debug(method(std::fmt::LowerHex::fmt)))]
    pub crc32: u32,
    pub compressed_size: u32,
    pub uncompressed_size: u32,
    pub file_name_length: u16,
    pub extra_field_length: u16,
    pub file_comment_length: u16,
    pub disk_number_start: u16,
    #[binwrite(with(binwrite_transform))]
    pub internal_file_attributes: InternalFileAttributes,
    pub external_file_attributes: u32,
    pub relative_header_offset: u32,
    #[educe(Debug(method(crate::utils::fmt_utf8)))]
    pub file_name: Vec<u8>,
    pub extra_fields: Vec<ExtraField>,
    /// only one of `extra_fields` and `extra_fields_raw` can be set
    #[educe(Debug(method(crate::utils::fmt_hex)))]
    pub extra_fields_raw: Vec<u8>,
    pub file_comment: Vec<u8>,

    #[binwrite(ignore)]
    pub zip64: Zip64ExtendedInfo,
}

impl CentralDirectoryHeader {
    pub const SIGNATURE: u32 = 0x02014b50;

    /// Set CDH field and ZIP64 field according to size
    pub fn set_offset(&mut self, offset: usize, force_zip64: bool) {
        if !force_zip64 {
            if let Ok(offset) = offset.try_into() {
                self.relative_header_offset = offset;
                self.zip64.relative_header_offset = None;
                return;
            }
        }
        self.relative_header_offset = u32::MAX;
        self.zip64.relative_header_offset = Some(offset as u64);
    }

    /// Finalize extra fields, add ZIP64 field
    pub fn finalize(&mut self) -> Result<()> {
        if !self.zip64.is_empty() {
            self.extra_fields.push(ExtraField {
                header_id: 0,
                size: 0,
                data: Box::new(self.zip64.clone()),
            });
        }

        if !self.extra_fields.is_empty() && !self.extra_fields_raw.is_empty() {
            bail!("extra_fields and extra_fields_raw cannot be set at the same time");
        }

        if self.extra_fields.is_empty() {
            self.extra_field_length = self
                .extra_fields_raw
                .len()
                .try_into()
                .context("Extra fields too long")?;
        } else {
            for field in &mut self.extra_fields {
                field.finalize()?;
            }

            self.extra_field_length = self
                .extra_fields
                .byte_count()
                .context("Failed to count extra fields")?
                .try_into()
                .context("Extra fields too long")?;
        }

        Ok(())
    }
}

impl From<&LocalFileHeader> for CentralDirectoryHeader {
    fn from(lfh: &LocalFileHeader) -> Self {
        Self {
            version_made_by: lfh.version_needed,
            version_needed: lfh.version_needed,
            general_purpose_flag: lfh.general_purpose_flag,
            compression_method: lfh.compression_method,
            last_mod: lfh.last_mod,
            crc32: lfh.crc32,
            compressed_size: lfh.compressed_size,
            uncompressed_size: lfh.uncompressed_size,
            file_name_length: lfh.file_name_length,
            extra_field_length: lfh.extra_field_length,
            file_name: lfh.file_name.clone(),
            extra_fields: lfh.extra_fields.clone(),
            extra_fields_raw: lfh.extra_fields_raw.clone(),
            zip64: lfh.zip64.clone(),
            ..Default::default()
        }
    }
}

impl From<&FileEntry> for CentralDirectoryHeader {
    fn from(fe: &FileEntry) -> Self {
        match &fe.dd {
            None => (&fe.lfh).into(),
            Some(dd) => Self {
                version_made_by: fe.lfh.version_needed,
                version_needed: fe.lfh.version_needed,
                general_purpose_flag: fe.lfh.general_purpose_flag,
                compression_method: fe.lfh.compression_method,
                last_mod: fe.lfh.last_mod,
                crc32: dd.crc32,
                compressed_size: dd.compressed_size.saturate(),
                uncompressed_size: dd.uncompressed_size.saturate(),
                file_name_length: fe.lfh.file_name_length,
                extra_field_length: fe.lfh.extra_field_length,
                file_name: fe.lfh.file_name.clone(),
                extra_fields: fe.lfh.extra_fields.clone(),
                extra_fields_raw: fe.lfh.extra_fields_raw.clone(),
                zip64: fe.lfh.zip64.clone(),
                ..Default::default()
            },
        }
    }
}
