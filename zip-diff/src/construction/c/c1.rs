use crate::utils::{testcase, testcase_arg};
use anyhow::Result;
use binwrite::BinWrite;
use zip_diff::dd::{DataDescriptor, U32or64};
use zip_diff::eocd::EndOfCentralDirectoryRecord;
use zip_diff::fields::CompressionMethod;
use zip_diff::lfh::LocalFileHeader;
use zip_diff::utils::{crc32_patch, BinCount};
use zip_diff::zip::{FileEntry, ZipArchive};

fn no_cdh_for_lfh() -> Result<ZipArchive> {
    let mut zip = ZipArchive::default();

    zip.add_simple("a", b"a")?;
    zip.add_simple("b", b"b")?;
    zip.finalize()?;

    let cdh = zip.cd.pop().unwrap();
    zip.eocdr.this_disk_cdh_count -= 1;
    zip.eocdr.total_cdh_count -= 1;
    zip.eocdr.size_of_cd -= cdh.byte_count()? as u32;

    Ok(zip)
}

fn truncating_lfh_stream_via_fake_records() -> Result<ZipArchive> {
    let mut zip = ZipArchive::default();

    zip.add_simple("a", b"a")?;
    zip.add_simple("b", b"b")?;
    zip.add_simple("c", b"c")?;
    zip.finalize()?;

    let eocdr = EndOfCentralDirectoryRecord {
        this_disk_cdh_count: 1,
        total_cdh_count: 1,
        size_of_cd: zip.cd[0].byte_count()?.try_into()?,
        offset_of_cd_wrt_starting_disk: zip.files[0].byte_count()?.try_into()?,
        ..Default::default()
    };

    zip.cd[0].write(&mut zip.files[0].data)?;
    eocdr.write(&mut zip.files[1].data)?;
    zip.finalize()?;

    Ok(zip)
}

fn truncating_lfh_stream_via_lfh_inside_comments() -> Result<ZipArchive> {
    let mut zip = ZipArchive::default();

    zip.add_simple("a", b"a")?;
    zip.add_simple("b", b"b")?;
    zip.add_simple("c", b"b")?;
    zip.finalize()?;

    let entry2 = zip.files.pop().unwrap();
    let entry1 = zip.files.pop().unwrap();

    let mut offset = zip.files.byte_count()?;
    zip.eocdr.offset_of_cd_wrt_starting_disk = offset.try_into()?;

    offset += zip.cd[0..1].byte_count()?;
    let cdh = &mut zip.cd[1];
    entry1.write(&mut cdh.file_comment)?;
    cdh.file_comment_length = cdh.file_comment.len().try_into()?;
    cdh.relative_header_offset = offset.try_into()?;

    let cdh = &mut zip.cd[2];
    offset += cdh.file_comment.len() + cdh.byte_count()? + zip.eocdr.byte_count()?;
    entry2.write(&mut zip.eocdr.zip_file_comment)?;
    zip.eocdr.zip_file_comment_length = zip.eocdr.zip_file_comment.len().try_into()?;
    cdh.relative_header_offset = offset.try_into()?;

    Ok(zip)
}

fn lfh_desync(overlap: bool) -> Result<ZipArchive> {
    let mut zip = ZipArchive::default();

    let mut buf = Vec::new();
    let entry = FileEntry::new("a", b"a", CompressionMethod::STORED, false, false)?;
    entry.write(&mut buf)?;

    zip.add_simple("junk", &buf)?;
    zip.add_simple("b", b"b")?;
    zip.finalize()?;

    let mut offset = LocalFileHeader {
        file_name: "junk".into(),
        ..Default::default()
    }
    .byte_count()?;
    let mut cd = Vec::new();
    entry.push_into_cd(&mut cd, &mut offset)?;

    if overlap {
        let mut offset = 0;
        zip.files[0].push_into_cd(&mut cd, &mut offset)?;
        zip.files[1].push_into_cd(&mut cd, &mut offset)?;
    }

    let eocdr = EndOfCentralDirectoryRecord {
        this_disk_cdh_count: cd.len().try_into()?,
        total_cdh_count: cd.len().try_into()?,
        size_of_cd: cd.byte_count()?.try_into()?,
        offset_of_cd_wrt_starting_disk: zip.byte_count()?.try_into()?,
        ..Default::default()
    };

    cd.write(&mut zip.eocdr.zip_file_comment)?;
    eocdr.write(&mut zip.eocdr.zip_file_comment)?;
    zip.eocdr.zip_file_comment_length = zip.eocdr.zip_file_comment.len().try_into()?;

    Ok(zip)
}

fn dd_pos(deflated: bool) -> Result<ZipArchive> {
    let file_a = FileEntry::new("a", b"a", CompressionMethod::STORED, false, false)?;
    let file_b = FileEntry::new("b", b"b", CompressionMethod::STORED, false, false)?;

    let junk1b = FileEntry::new(
        "junk1",
        b"",
        if deflated {
            CompressionMethod::DEFLATED
        } else {
            CompressionMethod::STORED
        },
        false,
        true,
    )?;

    let junk1a_bare = junk1b.clone();
    let junk2_bare = FileEntry::new("junk2", b"", CompressionMethod::STORED, false, false)?;
    let junk3_bare = FileEntry::new("junk3", b"", CompressionMethod::STORED, false, false)?;

    let junk2_len =
        junk1a_bare.dd.unwrap().byte_count()? + file_b.byte_count()? + junk3_bare.byte_count()? + 4;
    let junk2_lfh = LocalFileHeader {
        compressed_size: junk2_len as u32,
        uncompressed_size: junk2_len as u32,
        ..junk2_bare.lfh.clone()
    };

    let mut junk1a_data = junk1a_bare.data;
    junk1b.dd.as_ref().unwrap().write(&mut junk1a_data)?;
    file_b.write(&mut junk1a_data)?;
    junk2_lfh.write(&mut junk1a_data)?;

    let junk1a_dd = DataDescriptor {
        signature: Some(DataDescriptor::SIGNATURE),
        crc32: crc32fast::hash(&junk1a_data),
        compressed_size: U32or64::U32(junk1a_data.len() as u32),
        uncompressed_size: U32or64::U32(junk1a_data.len() as u32),
    };

    let mut zip_b_tmp = ZipArchive::default();
    zip_b_tmp.files.push(junk1b.clone());
    zip_b_tmp.files.push(file_b);
    zip_b_tmp.files.push(junk2_bare);
    zip_b_tmp.finalize()?;
    let junk3_len = 4 + zip_b_tmp.cd.byte_count()? + zip_b_tmp.eocdr.byte_count()? + 4;
    let junk3_lfh = LocalFileHeader {
        compressed_size: junk3_len as u32,
        uncompressed_size: junk3_len as u32,
        ..junk3_bare.lfh.clone()
    };

    let mut junk2_data = Vec::new();
    junk1a_dd.write(&mut junk2_data)?;
    file_a.write(&mut junk2_data)?;
    junk3_lfh.write(&mut junk2_data)?;
    let junk2_patch = crc32_patch(&junk2_data, junk2_lfh.crc32);
    junk2_patch.write(&mut junk2_data)?;

    let junk1a = FileEntry {
        lfh: junk1a_bare.lfh,
        data: junk1a_data,
        dd: Some(junk1a_dd),
    };

    let junk2 = FileEntry {
        lfh: junk2_lfh,
        data: junk2_data,
        dd: None,
    };

    let mut zip_a_tmp = ZipArchive::default();
    zip_a_tmp.files.push(junk1a.clone());
    zip_a_tmp.files.push(file_a);
    zip_a_tmp.files.push(junk3_bare);
    zip_a_tmp.finalize()?;

    let mut zip_b = zip_b_tmp;
    *zip_b.files.last_mut().unwrap() = junk2;
    zip_b.finalize()?;
    zip_b.eocdr.zip_file_comment_length =
        (4 + zip_a_tmp.cd.byte_count()? + zip_a_tmp.eocdr.byte_count()?) as u16;

    let mut junk3_data = Vec::new();
    junk2_patch.write(&mut junk3_data)?;
    zip_b.cd.write(&mut junk3_data)?;
    zip_b.eocdr.write(&mut junk3_data)?;
    let junk3_patch = crc32_patch(&junk3_data, junk3_lfh.crc32);
    junk3_patch.write(&mut junk3_data)?;

    let junk3 = FileEntry {
        lfh: junk3_lfh,
        data: junk3_data,
        dd: None,
    };

    let mut zip_a = zip_a_tmp;
    *zip_a.files.last_mut().unwrap() = junk3;
    zip_a.finalize()?;
    zip_a.cd[0].compression_method = CompressionMethod::STORED;

    Ok(zip_a)
}

pub fn main() -> Result<()> {
    testcase(no_cdh_for_lfh)?;
    testcase(truncating_lfh_stream_via_fake_records)?;
    testcase(truncating_lfh_stream_via_lfh_inside_comments)?;
    [false, true]
        .iter()
        .try_for_each(|overlap| testcase_arg(lfh_desync, *overlap))?;
    [false, true]
        .iter()
        .try_for_each(|deflate| testcase_arg(dd_pos, *deflate))?;
    Ok(())
}
