use crate::utils::{testcase, testcase_arg};
use anyhow::Result;
use bitflags::bitflags;
use zip_diff::dd::{DataDescriptor, U32or64};
use zip_diff::extra::Zip64ExtendedInfo;
use zip_diff::fields::{CompressionMethod, GeneralPurposeFlag};
use zip_diff::utils::crc32_patch;
use zip_diff::zip::ZipArchive;

const DATA: &[u8] = b"test";

#[derive(Clone, Copy)]
struct LfhCdh(u8);

bitflags! {
    impl LfhCdh: u8 {
        const Deflated = 1 << 0;
        const LfhCompressed = 1 << 1;
        const LfhUncompressed = 1 << 2;
        const CdhCompressed = 1 << 3;
        const CdhUncompressed = 1 << 4;
    }
}

#[derive(Clone, Copy)]
struct DataDescriptorFlags(u8);

bitflags! {
    impl DataDescriptorFlags: u8 {
        const CompressedZero = 1 << 0;
        const UncompressedZero = 1 << 1;
        const Size64 = 1 << 2;
    }
}

#[derive(Clone, Copy)]
struct Zip64Flags(u8);

bitflags! {
    impl Zip64Flags: u8 {
        const CompressedSize = 1 << 0;
        const UncompressedSize = 1 << 1;
    }
}

struct Args {
    lfh_cdh_flags: LfhCdh,
    lfh_zip64: Zip64Flags,
    cdh_zip64: Zip64Flags,
    dd_flags: Option<DataDescriptorFlags>,
}

fn size_confusion(args: Args) -> Result<ZipArchive> {
    let mut zip = ZipArchive::default();

    let mut data = Vec::from(DATA);
    let patch = crc32_patch(&data, 0);
    data.extend(patch.to_le_bytes());

    let compression = if args.lfh_cdh_flags.contains(LfhCdh::Deflated) {
        CompressionMethod::DEFLATED
    } else {
        CompressionMethod::STORED
    };

    zip.add_file("test", &data, compression, false, false)?;
    zip.finalize()?;

    if let Some(flags) = args.dd_flags {
        let lfh = &mut zip.files[0].lfh;
        let cdh = &mut zip.cd[0];

        let compressed_size = if flags.contains(DataDescriptorFlags::CompressedZero) {
            0
        } else {
            lfh.compressed_size
        };

        let uncompressed_size = if flags.contains(DataDescriptorFlags::UncompressedZero) {
            0
        } else {
            lfh.uncompressed_size
        };

        let (compressed_size, uncompressed_size) = if flags.contains(DataDescriptorFlags::Size64) {
            (
                U32or64::U64(compressed_size.into()),
                U32or64::U64(uncompressed_size.into()),
            )
        } else {
            (
                U32or64::U32(compressed_size),
                U32or64::U32(uncompressed_size),
            )
        };

        let dd = DataDescriptor {
            compressed_size,
            uncompressed_size,
            ..Default::default()
        };

        lfh.general_purpose_flag
            .insert(GeneralPurposeFlag::DataDescriptor);
        cdh.general_purpose_flag
            .insert(GeneralPurposeFlag::DataDescriptor);
        zip.files[0].dd = Some(dd);
    }

    let lfh = &mut zip.files[0].lfh;
    let cdh = &mut zip.cd[0];

    if args.lfh_cdh_flags.contains(LfhCdh::LfhCompressed) {
        lfh.compressed_size = 0;
    }
    if args.lfh_cdh_flags.contains(LfhCdh::LfhUncompressed) {
        lfh.uncompressed_size = 0;
    }
    if args.lfh_cdh_flags.contains(LfhCdh::CdhCompressed) {
        cdh.compressed_size = 0;
    }
    if args.lfh_cdh_flags.contains(LfhCdh::CdhUncompressed) {
        cdh.uncompressed_size = 0;
    }

    if !args.lfh_zip64.is_empty() {
        let compressed_size = if args.lfh_zip64.contains(Zip64Flags::CompressedSize) {
            let size = lfh.compressed_size;
            lfh.compressed_size = u32::MAX;
            Some(size.into())
        } else {
            None
        };
        let original_size = if args.lfh_zip64.contains(Zip64Flags::UncompressedSize) {
            let size = lfh.uncompressed_size;
            lfh.uncompressed_size = u32::MAX;
            Some(size.into())
        } else {
            None
        };
        let zip64 = Zip64ExtendedInfo {
            compressed_size,
            original_size,
            ..Default::default()
        };
        lfh.extra_fields.push(zip64.into());
    }

    if !args.cdh_zip64.is_empty() {
        let compressed_size = if args.cdh_zip64.contains(Zip64Flags::CompressedSize) {
            let size = cdh.compressed_size;
            cdh.compressed_size = u32::MAX;
            Some(size.into())
        } else {
            None
        };
        let original_size = if args.cdh_zip64.contains(Zip64Flags::UncompressedSize) {
            let size = cdh.uncompressed_size;
            cdh.uncompressed_size = u32::MAX;
            Some(size.into())
        } else {
            None
        };
        let zip64 = Zip64ExtendedInfo {
            compressed_size,
            original_size,
            ..Default::default()
        };
        cdh.extra_fields.push(zip64.into());
    }

    zip.set_offsets(0)?;

    Ok(zip)
}

fn multiple_zip64() -> Result<ZipArchive> {
    let mut zip = ZipArchive::default();
    zip.add_file("test", DATA, CompressionMethod::STORED, true, false)?;
    zip.finalize()?;
    let zip64 = Zip64ExtendedInfo {
        original_size: Some(0),
        compressed_size: Some(0),
        relative_header_offset: None,
        disk_start_number: None,
    };
    zip.files[0].lfh.extra_fields.push(zip64.clone().into());
    zip.cd[0].extra_fields.push(zip64.into());
    zip.set_offsets(0)?;
    Ok(zip)
}

pub fn main() -> Result<()> {
    for i in 0..32 {
        let lfh_cdh_flags = LfhCdh::from_bits_truncate(i);
        for i in 0..=8 {
            let dd_flags = if i == 8 {
                None
            } else {
                Some(DataDescriptorFlags::from_bits_truncate(i))
            };
            for i in 0..4 {
                let lfh_zip64 = Zip64Flags::from_bits_truncate(i);
                for i in 0..4 {
                    let cdh_zip64 = Zip64Flags::from_bits_truncate(i);
                    let args = Args {
                        lfh_cdh_flags,
                        dd_flags,
                        lfh_zip64,
                        cdh_zip64,
                    };
                    testcase_arg(size_confusion, args)?;
                }
            }
        }
    }
    testcase(multiple_zip64)?;
    Ok(())
}
