use crate::utils::{testcase, testcase_arg};
use anyhow::Result;
use binwrite::BinWrite;
use bitflags::bitflags;
use zip_diff::eocd::{
    EndOfCentralDirectoryRecord, Zip64EndOfCentralDirectoryLocator,
    Zip64EndOfCentralDirectoryRecord, Zip64ExtensibleDataSector,
};
use zip_diff::extra::Zip64ExtendedInfo;
use zip_diff::utils::BinCount;
use zip_diff::zip::ZipArchive;

struct Zip64Flags(u8);

bitflags! {
    impl Zip64Flags: u8 {
        const DiskNumberFF = 1 << 0;
        const CdhCountFF = 1 << 1;
        const CdSizeFF = 1 << 2;
        const CdOffsetFF = 1 << 3;
        const EocdlGap = 1 << 4;
        const MoreFilesInZip64 = 1 << 5;
    }
}

fn use_zip64_eocdr(flags: Zip64Flags) -> Result<ZipArchive> {
    let mut zip1 = ZipArchive::default();
    zip1.add_simple("a", b"a")?;
    if !flags.contains(Zip64Flags::MoreFilesInZip64) {
        zip1.add_simple("b", b"b")?;
    }
    zip1.finalize()?;

    let mut zip2 = ZipArchive::default();
    zip2.add_simple("c", b"c")?;
    if flags.contains(Zip64Flags::MoreFilesInZip64) {
        zip2.add_simple("d", b"d")?;
    }
    zip2.finalize()?;
    zip2.set_offsets(zip1.files.byte_count()? + zip1.cd.byte_count()?)?;
    zip2.set_eocd(true)?;

    let cdh = zip1.cd.last_mut().unwrap();
    zip2.files.write(&mut cdh.file_comment)?;
    zip2.cd.write(&mut cdh.file_comment)?;
    zip2.zip64_eocdr.unwrap().write(&mut cdh.file_comment)?;
    zip2.zip64_eocdl.unwrap().write(&mut cdh.file_comment)?;
    if flags.contains(Zip64Flags::EocdlGap) {
        0u8.write(&mut cdh.file_comment)?;
    }
    cdh.file_comment_length = cdh.file_comment.len().try_into()?;

    zip1.set_eocd(false)?;

    if flags.contains(Zip64Flags::DiskNumberFF) {
        zip1.eocdr.number_of_this_disk = u16::MAX;
        zip1.eocdr.start_of_cd_disk_number = u16::MAX;
    }

    if flags.contains(Zip64Flags::CdhCountFF) {
        zip1.eocdr.this_disk_cdh_count = u16::MAX;
        zip1.eocdr.total_cdh_count = u16::MAX;
    }

    if flags.contains(Zip64Flags::CdSizeFF) {
        zip1.eocdr.size_of_cd = u32::MAX;
    }

    if flags.contains(Zip64Flags::CdOffsetFF) {
        zip1.eocdr.offset_of_cd_wrt_starting_disk = u32::MAX;
    }

    Ok(zip1)
}

fn eocdl_or_search() -> Result<ZipArchive> {
    let mut zip1 = ZipArchive::default();
    zip1.add_simple("a", b"a")?;
    zip1.finalize()?;
    zip1.set_eocd(true)?;

    let mut zip2 = ZipArchive::default();
    zip2.add_simple("b", b"b")?;
    zip2.finalize()?;
    zip2.set_offsets(zip1.files.byte_count()? + zip1.cd.byte_count()?)?;
    zip2.set_eocd(true)?;

    // hide ZIP64 EOCDR of zip1 in the ZIP64 EOCDR extensible data sector of zip2
    let zip64_eocdr_size = zip1.zip64_eocdr.as_ref().unwrap().byte_count()?;
    let zip64_eocdr_2 = zip2.zip64_eocdr.as_mut().unwrap();
    let extensible_header = Zip64ExtensibleDataSector {
        header_id: 0x1337, // an unknown ID
        size: zip64_eocdr_size.try_into()?,
        data: Box::new(Zip64ExtendedInfo::default()), // empty data
    };
    zip64_eocdr_2.size += u64::try_from(extensible_header.byte_count()? + zip64_eocdr_size)?;
    zip64_eocdr_2.extensible_data_sector.push(extensible_header);

    let cdh = &mut zip1.cd[0];
    zip2.files.write(&mut cdh.file_comment)?;
    zip2.cd.write(&mut cdh.file_comment)?;
    zip2.zip64_eocdr
        .as_ref()
        .unwrap()
        .write(&mut cdh.file_comment)?;
    cdh.file_comment_length = cdh.file_comment.len().try_into()?;

    zip1.set_eocd(true)?;
    zip1.zip64_eocdl.as_mut().unwrap().zip64_eocdr_offset -=
        u64::try_from(zip2.zip64_eocdr.unwrap().byte_count()?)?;

    Ok(zip1)
}

struct CdhCountFlags(u8);

bitflags! {
    impl CdhCountFlags: u8 {
        const ThisDiskCount = 1 << 0;
        const TotalCount = 1 << 1;
        const CdSize = 1 << 2;
    }
}

fn cdh_count(flags: CdhCountFlags) -> Result<ZipArchive> {
    let mut zip = ZipArchive::default();

    zip.add_simple("a", b"a")?;
    zip.add_simple("b", b"b")?;
    zip.finalize()?;
    zip.set_eocd(true)?;

    let eocdr = zip.zip64_eocdr.as_mut().unwrap();

    if flags.contains(CdhCountFlags::ThisDiskCount) {
        eocdr.this_disk_cdh_count -= 1;
    }

    if flags.contains(CdhCountFlags::TotalCount) {
        eocdr.total_cdh_count -= 1;
    }

    if flags.contains(CdhCountFlags::CdSize) {
        eocdr.size_of_cd = zip.cd[0].byte_count()?.try_into()?;
    }

    Ok(zip)
}

fn cd_offset(adjust_zip64_offset: bool) -> Result<Vec<u8>> {
    let zip = super::c4::cd_offset()?;
    let eocdr = zip.eocdr;

    let mut buf = Vec::new();
    zip.groups.write(&mut buf)?;

    let mut zip64_eocdr = Zip64EndOfCentralDirectoryRecord {
        this_disk_cdh_count: eocdr.this_disk_cdh_count.into(),
        total_cdh_count: eocdr.this_disk_cdh_count.into(),
        size_of_cd: eocdr.size_of_cd.into(),
        offset_of_cd_wrt_starting_disk: eocdr.offset_of_cd_wrt_starting_disk.into(),
        ..Default::default()
    };
    zip64_eocdr.finalize()?;

    let zip64_offset = if adjust_zip64_offset {
        zip.groups[0..=1].byte_count()?
    } else {
        buf.len()
    };

    let eocdl = Zip64EndOfCentralDirectoryLocator::from_offset(zip64_offset.try_into()?);
    let eocdr = EndOfCentralDirectoryRecord::all_ff();

    zip64_eocdr.write(&mut buf)?;
    eocdl.write(&mut buf)?;
    eocdr.write(&mut buf)?;

    Ok(buf)
}

pub fn main() -> Result<()> {
    (0..64).try_for_each(|i| testcase_arg(use_zip64_eocdr, Zip64Flags::from_bits_truncate(i)))?;
    testcase(eocdl_or_search)?;
    (1..8).try_for_each(|i| testcase_arg(cdh_count, CdhCountFlags::from_bits_truncate(i)))?;
    testcase_arg(cd_offset, false)?;
    testcase_arg(cd_offset, true)?;
    Ok(())
}
