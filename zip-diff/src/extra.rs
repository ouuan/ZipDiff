use crate::utils::{binwrite_option, binwrite_transform, BinWriteTransform};
use anyhow::{Context, Result};
use binwrite::BinWrite;
use bitflags::bitflags;
use downcast_rs::{impl_downcast, Downcast};
use dyn_clone::{clone_trait_object, DynClone};
use std::fmt::Debug;

#[derive(BinWrite, Clone, Debug)]
pub struct ExtraField {
    pub header_id: u16,
    pub size: u16,
    #[binwrite(with(binwrite_transform))]
    pub data: Box<dyn ExtraFieldType>,
}

impl ExtraField {
    pub fn finalize(&mut self) -> Result<()> {
        self.header_id = self.data.header_id();
        self.size = self
            .data
            .binary_encode()
            .context("Failed to count extra field size")?
            .len()
            .try_into()
            .context("Extra field too long")?;
        Ok(())
    }
}

impl<T: ExtraFieldType> From<T> for ExtraField {
    fn from(data: T) -> Self {
        Self {
            header_id: 0,
            size: 0,
            data: Box::new(data),
        }
    }
}

pub trait ExtraFieldType: BinaryEncode + Debug + DynClone + Downcast + Send + Sync {
    // a function is used instead of an associated const to make it object-safe
    fn header_id(&self) -> u16;
}

impl_downcast!(ExtraFieldType);
clone_trait_object!(ExtraFieldType);

#[derive(BinWrite, Clone, Default, Debug)]
pub struct Zip64ExtendedInfo {
    #[binwrite(with(binwrite_option))]
    pub original_size: Option<u64>,
    #[binwrite(with(binwrite_option))]
    pub compressed_size: Option<u64>,
    #[binwrite(with(binwrite_option))]
    pub relative_header_offset: Option<u64>,
    #[binwrite(with(binwrite_option))]
    pub disk_start_number: Option<u32>,
}

impl ExtraFieldType for Zip64ExtendedInfo {
    fn header_id(&self) -> u16 {
        1
    }
}

impl Zip64ExtendedInfo {
    pub fn is_empty(&self) -> bool {
        self.original_size.is_none()
            && self.compressed_size.is_none()
            && self.relative_header_offset.is_none()
            && self.disk_start_number.is_none()
    }
}

#[derive(Clone, Copy, Default, Debug)]
pub struct PatchDescriptorFlag(u32);

bitflags! {
    impl PatchDescriptorFlag: u32 {
        const AutoDetection = 1 << 0;
        const SelfPatch = 1 << 1;
        const ActionAdd = 1 << 4;
        const ActionDelete = 2 << 4;
        const ActionPatch = 3 << 4;
        const ReactionToAbsentSkip = 1 << 8;
        const ReactionToAbsentIgnore = 2 << 8;
        const ReactionToAbsentFail = 3 << 8;
        const ReactionToNewerSkip = 1 << 10;
        const ReactionToNewerIgnore = 2 << 10;
        const ReactionToNewerFail = 3 << 10;
        const ReactionToUnknownSkip = 1 << 12;
        const ReactionToUnknownIgnore = 2 << 12;
        const ReactionToUnknownFail = 3 << 12;
        const _ = !0;
    }
}

impl BinWriteTransform for PatchDescriptorFlag {
    type Type = u32;
    fn binwrite_transform(&self) -> std::io::Result<Self::Type> {
        Ok(self.0)
    }
}

#[derive(BinWrite, Clone, Default, Debug)]
pub struct PatchDescriptor {
    pub version: u16,
    #[binwrite(with(binwrite_transform))]
    pub flags: PatchDescriptorFlag,
    pub old_size: u32,
    pub old_crc: u32,
    pub new_size: u32,
    pub new_crc: u32,
}

impl ExtraFieldType for PatchDescriptor {
    fn header_id(&self) -> u16 {
        0xf
    }
}

#[derive(BinWrite, Clone, Default, Debug)]
pub struct InfoZipUnicodePath {
    pub version: u8,
    pub name_crc32: u32,
    pub unicode_name: String,
}

impl ExtraFieldType for InfoZipUnicodePath {
    fn header_id(&self) -> u16 {
        0x7075
    }
}

impl InfoZipUnicodePath {
    pub fn new(unicode_name: String, name: &str) -> Self {
        Self {
            version: 1,
            name_crc32: crc32fast::hash(name.as_bytes()),
            unicode_name,
        }
    }
}

// BinWrite is not object-safe.
// The following is to make BinWrite with Box<dyn BinWrite> possible.

pub trait BinaryEncode {
    fn binary_encode(&self) -> std::io::Result<Vec<u8>>;
}

impl<T: BinWrite + ?Sized> BinaryEncode for T {
    fn binary_encode(&self) -> std::io::Result<Vec<u8>> {
        let mut bytes = Vec::new();
        self.write(&mut bytes)?;
        Ok(bytes)
    }
}

impl BinWriteTransform for Box<dyn ExtraFieldType> {
    type Type = Vec<u8>;

    fn binwrite_transform(&self) -> std::io::Result<Self::Type> {
        self.binary_encode()
    }
}
