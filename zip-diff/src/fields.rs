use crate::utils::BinWriteTransform;
use binwrite::BinWrite;
use bitflags::bitflags;
use chrono::{DateTime, Datelike, Timelike, Utc};

#[derive(Clone, Copy, Default, Debug)]
pub struct GeneralPurposeFlag(u16);

bitflags! {
    impl GeneralPurposeFlag: u16 {
        const Encrypted = 1 << 0;
        const Compression1 = 1 << 1;
        const Compression2 = 1 << 2;
        const DataDescriptor = 1 << 3;
        const PatchData = 1 << 5;
        const StrongEncryption = 1 << 6;
        const LanguageEncoding = 1 << 11;
        const EncryptedCentralDirectory = 1 << 13;
        const _ = !0;
    }
}

impl BinWriteTransform for GeneralPurposeFlag {
    type Type = u16;
    fn binwrite_transform(&self) -> std::io::Result<Self::Type> {
        Ok(self.0)
    }
}

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct CompressionMethod(pub u16);

impl CompressionMethod {
    pub const STORED: Self = Self(0);
    pub const SHRUNK: Self = Self(1);
    pub const REDUCED1: Self = Self(2);
    pub const REDUCED2: Self = Self(3);
    pub const REDUCED3: Self = Self(4);
    pub const REDUCED4: Self = Self(5);
    pub const IMPLODED: Self = Self(6);
    pub const DEFLATED: Self = Self(8);
    pub const DEFLATE64: Self = Self(9);
    pub const BZIP2: Self = Self(12);
    pub const LZMA: Self = Self(14);
    pub const ZSTD: Self = Self(93);
    pub const MP3: Self = Self(94);
    pub const XZ: Self = Self(95);
    pub const JPEG: Self = Self(96);
}

impl BinWriteTransform for CompressionMethod {
    type Type = u16;
    fn binwrite_transform(&self) -> std::io::Result<Self::Type> {
        Ok(self.0)
    }
}

#[derive(Clone, Copy, Default, Debug)]
pub struct InternalFileAttributes(u16);

bitflags! {
    impl InternalFileAttributes: u16 {
        const TextFile = 1 << 0;
        const RecordLengthControl = 1 << 2;
        const _ = !0;
    }
}

impl BinWriteTransform for InternalFileAttributes {
    type Type = u16;
    fn binwrite_transform(&self) -> std::io::Result<Self::Type> {
        Ok(self.0)
    }
}

#[derive(BinWrite, Clone, Copy, Debug)]
pub struct DosDateTime {
    pub time: u16,
    pub date: u16,
}

impl DosDateTime {
    pub fn new(time: u16, date: u16) -> Self {
        Self { time, date }
    }
}

impl From<DateTime<Utc>> for DosDateTime {
    fn from(dt: DateTime<Utc>) -> Self {
        let date = ((((dt.year() - 1980) as u32) << 9) | (dt.month() << 5) | dt.day()) as u16;
        let time = ((dt.hour() << 11) | (dt.minute() << 5) | (dt.second() / 2)) as u16;
        DosDateTime { date, time }
    }
}

impl Default for DosDateTime {
    fn default() -> Self {
        Utc::now().into()
    }
}
