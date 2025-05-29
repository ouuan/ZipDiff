use crate::extra::{ExtraField, Zip64ExtendedInfo};
use crate::fields::*;
use crate::utils::{binwrite_transform, BinCount};
use anyhow::{bail, Context, Result};
use binwrite::BinWrite;
use educe::Educe;

#[derive(BinWrite, Clone, Educe)]
#[educe(Debug, Default)]
pub struct LocalFileHeader {
    #[educe(Default = Self::SIGNATURE)]
    #[educe(Debug(method(std::fmt::LowerHex::fmt)))]
    pub signature: u32,
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
    #[educe(Debug(method(crate::utils::fmt_utf8)))]
    pub file_name: Vec<u8>,
    pub extra_fields: Vec<ExtraField>,
    /// only one of `extra_fields` and `extra_fields_raw` can be set
    #[educe(Debug(method(crate::utils::fmt_hex)))]
    pub extra_fields_raw: Vec<u8>,

    #[binwrite(ignore)]
    pub zip64: Zip64ExtendedInfo,
    #[binwrite(ignore)]
    pub keep_empty_zip64: bool,
}

impl LocalFileHeader {
    pub const SIGNATURE: u32 = 0x04034b50;

    /// Set LFH field and ZIP64 field according to size
    pub fn set_compressed_size(&mut self, size: usize, force_zip64: bool) {
        if !force_zip64 {
            if let Ok(size) = size.try_into() {
                self.compressed_size = size;
                self.zip64.compressed_size = None;
                return;
            }
        }
        self.compressed_size = u32::MAX;
        self.zip64.compressed_size = Some(size as u64);
    }

    /// Set LFH field and ZIP64 field according to size
    pub fn set_uncompressed_size(&mut self, size: usize, force_zip64: bool) {
        if !force_zip64 {
            if let Ok(size) = size.try_into() {
                self.uncompressed_size = size;
                self.zip64.original_size = None;
                return;
            }
        }
        self.uncompressed_size = u32::MAX;
        self.zip64.original_size = Some(size as u64);
    }

    pub fn set_file_name(&mut self, file_name: &str) -> Result<()> {
        file_name.as_bytes().clone_into(&mut self.file_name);
        self.file_name_length = self.file_name.len().try_into()?;
        Ok(())
    }

    /// Finalize extra fields, add ZIP64 field
    pub fn finalize(&mut self) -> Result<()> {
        if self.keep_empty_zip64 || !self.zip64.is_empty() {
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
