use crate::utils::{testcase, EntryGroup, CRC32A, CRC32B};
use anyhow::Result;
use binwrite::BinWrite;
use zip_diff::cdh::CentralDirectoryHeader;
use zip_diff::eocd::EndOfCentralDirectoryRecord;
use zip_diff::lfh::LocalFileHeader;
use zip_diff::utils::BinCount;
use zip_diff::zip::ZipArchive;

#[derive(BinWrite)]
pub struct CdOffsetZip {
    pub groups: Vec<EntryGroup>,
    pub eocdr: EndOfCentralDirectoryRecord,
}

pub fn cd_offset() -> Result<CdOffsetZip> {
    let mut zip = ZipArchive::default();

    zip.add_simple("stream", b"a")?;
    zip.add_simple("eocdr", b"a")?;

    const FILENAME: &str = "adjac";
    let cd_size = CentralDirectoryHeader::from(&zip.files[1].lfh).byte_count()?;
    let lfh_size = LocalFileHeader {
        file_name: FILENAME.into(),
        ..Default::default()
    }
    .byte_count()?;

    let content_width = cd_size - lfh_size;
    zip.add_simple(FILENAME, format!("{CRC32A:A<0$}", content_width).as_bytes())?;
    zip.add_simple(FILENAME, format!("{CRC32B:A<0$}", content_width).as_bytes())?;

    zip.finalize()?;

    // This is required for correct LFH offset adjustment
    // This is ensured by adjusting the file name length
    assert_eq!(cd_size, zip.files[3].byte_count()?);
    // This is required so that the CD size in EOCDR is correct for both central directories
    assert_eq!(cd_size, zip.cd[2].byte_count()?);

    {
        zip.cd[3].relative_header_offset = zip.cd[2].relative_header_offset;
        // Make sure the CDHs match, as they will have the same CDH but different LFH offsets
        // In particular, the filename and CRC32 must be the same
        let mut tmp1 = Vec::new();
        zip.cd[2].write(&mut tmp1)?;
        let mut tmp2 = Vec::new();
        zip.cd[3].write(&mut tmp2)?;
        assert_eq!(tmp1, tmp2);
    }

    // for streaming mode parsers
    let group1 = EntryGroup {
        files: vec![zip.files[0].clone()],
        cd: vec![zip.cd[0].clone()],
    };

    let mut group2 = EntryGroup {
        // first file for parsers that use the CDH at the offset in EOCDR
        // second for parsers that use the adjacent central directory but does not adjust LFH offsets
        files: zip.files[1..=2].to_vec(),
        cd: vec![zip.cd[1].clone()],
    };
    group2.cd[0].relative_header_offset = group1.byte_count()?.try_into()?;

    // for parsers that use the adjacent central directory and adjusts LFH offsets accordingly
    let mut group3 = EntryGroup {
        files: vec![zip.files[3].clone()],
        cd: vec![zip.cd[2].clone()],
    };
    group3.cd[0].relative_header_offset =
        group2.cd[0].relative_header_offset + u32::try_from(zip.files[1].byte_count()?)?;

    let eocdr = EndOfCentralDirectoryRecord {
        this_disk_cdh_count: 1,
        total_cdh_count: 1,
        size_of_cd: cd_size.try_into()?,
        offset_of_cd_wrt_starting_disk: group2.cd[0].relative_header_offset
            + u32::try_from(group2.files.byte_count()?)?,
        ..Default::default()
    };

    Ok(CdOffsetZip {
        groups: vec![group1, group2, group3],
        eocdr,
    })
}

pub fn main() -> Result<()> {
    testcase(cd_offset)?;
    Ok(())
}
