use crate::utils::{testcase, CRC32A, CRC32B};
use anyhow::Result;
use zip_diff::extra::InfoZipUnicodePath;
use zip_diff::fields::GeneralPurposeFlag;
use zip_diff::zip::ZipArchive;

const DATA: &[u8] = b"test";

fn lfh_cdh() -> Result<ZipArchive> {
    let mut zip = ZipArchive::default();
    zip.add_simple("lfh", DATA)?;
    zip.finalize()?;
    zip.cd[0].file_name = "cdh".into();
    Ok(zip)
}

fn up_lfh_cdh() -> Result<ZipArchive> {
    let mut zip = ZipArchive::default();
    zip.add_simple("original", DATA)?;
    let lfh = &mut zip.files[0].lfh;
    let name_crc32 = crc32fast::hash(&lfh.file_name);
    let up = InfoZipUnicodePath {
        version: 1,
        name_crc32,
        unicode_name: "lfh".into(),
    };
    lfh.extra_fields.push(up.into());
    zip.finalize()?;
    let cd_up: &mut InfoZipUnicodePath = zip.cd[0].extra_fields[0].data.downcast_mut().unwrap();
    cd_up.unicode_name = "cdh".into();
    Ok(zip)
}

fn up_version() -> Result<ZipArchive> {
    let mut zip = ZipArchive::default();

    zip.add_simple("v0-original", DATA)?;
    let lfh = &mut zip.files[0].lfh;
    let name_crc32 = crc32fast::hash(&lfh.file_name);
    let up = InfoZipUnicodePath {
        version: 0,
        name_crc32,
        unicode_name: "v0-up".into(),
    };
    lfh.extra_fields.push(up.into());

    zip.add_simple("v2-original", DATA)?;
    let lfh = &mut zip.files[1].lfh;
    let name_crc32 = crc32fast::hash(&lfh.file_name);
    let up = InfoZipUnicodePath {
        version: 2,
        name_crc32,
        unicode_name: "v2-up".into(),
    };
    lfh.extra_fields.push(up.into());

    zip.finalize()?;
    Ok(zip)
}

fn up_incorrect_crc32() -> Result<ZipArchive> {
    let mut zip = ZipArchive::default();

    zip.add_simple("original", DATA)?;
    let lfh = &mut zip.files[0].lfh;
    let up = InfoZipUnicodePath {
        version: 1,
        name_crc32: 0,
        unicode_name: "up".into(),
    };
    lfh.extra_fields.push(up.into());

    zip.finalize()?;
    Ok(zip)
}

fn up_crc32_source() -> Result<ZipArchive> {
    let mut zip = ZipArchive::default();

    zip.add_simple("1-original", DATA)?;
    let lfh = &mut zip.files[0].lfh;
    let name_crc32 = crc32fast::hash(&lfh.file_name);
    let up1 = InfoZipUnicodePath {
        version: 1,
        name_crc32,
        unicode_name: "1-up1".into(),
    };
    let up2 = InfoZipUnicodePath {
        version: 1,
        name_crc32,
        unicode_name: "1-up2".into(),
    };
    lfh.extra_fields.push(up1.into());
    lfh.extra_fields.push(up2.into());

    zip.add_simple("2-original", DATA)?;
    let lfh = &mut zip.files[1].lfh;
    let name_crc32 = crc32fast::hash(&lfh.file_name);
    let up1 = InfoZipUnicodePath {
        version: 1,
        name_crc32,
        unicode_name: "2-up1".into(),
    };
    let name_crc32 = crc32fast::hash(up1.unicode_name.as_bytes());
    let up2 = InfoZipUnicodePath {
        version: 1,
        name_crc32,
        unicode_name: "2-up2".into(),
    };
    lfh.extra_fields.push(up1.into());
    lfh.extra_fields.push(up2.into());

    zip.finalize()?;
    Ok(zip)
}

fn up_invalid() -> Result<ZipArchive> {
    let mut zip = ZipArchive::default();

    zip.add_simple("original", DATA)?;
    let lfh = &mut zip.files[0].lfh;
    let name_crc32 = crc32fast::hash(&lfh.file_name);
    let up1 = InfoZipUnicodePath {
        version: 1,
        name_crc32,
        unicode_name: "up-valid".into(),
    };
    // invalid for both version and CRC32
    let up2 = InfoZipUnicodePath {
        version: 2,
        name_crc32: 0,
        unicode_name: "up-invalid".into(),
    };
    lfh.extra_fields.push(up1.into());
    lfh.extra_fields.push(up2.into());

    // Same CRC32 to make sure CRC32 check in up3 does not fail regardless of the filename source
    zip.add_simple(&format!("{CRC32A}{CRC32A}"), DATA)?;
    let lfh = &mut zip.files[1].lfh;
    let name_crc32 = crc32fast::hash(&lfh.file_name);
    let up1 = InfoZipUnicodePath {
        version: 1,
        name_crc32,
        unicode_name: format!("{CRC32A}{CRC32B}"),
    };
    let up2 = InfoZipUnicodePath {
        version: 2,
        name_crc32: 0,
        unicode_name: format!("{CRC32B}{CRC32A}"),
    };
    let up3 = InfoZipUnicodePath {
        version: 1,
        name_crc32,
        unicode_name: format!("{CRC32B}{CRC32B}"),
    };
    lfh.extra_fields.push(up1.into());
    lfh.extra_fields.push(up2.into());
    lfh.extra_fields.push(up3.into());

    zip.finalize()?;
    Ok(zip)
}

fn up_efs() -> Result<ZipArchive> {
    let mut zip = ZipArchive::default();

    zip.add_simple("original", DATA)?;
    let lfh = &mut zip.files[0].lfh;
    lfh.general_purpose_flag
        .insert(GeneralPurposeFlag::LanguageEncoding);
    let name_crc32 = crc32fast::hash(&lfh.file_name);
    let up = InfoZipUnicodePath {
        version: 1,
        name_crc32,
        unicode_name: "up".into(),
    };
    lfh.extra_fields.push(up.into());

    zip.finalize()?;
    Ok(zip)
}

pub fn main() -> Result<()> {
    testcase(lfh_cdh)?;
    testcase(up_lfh_cdh)?;
    testcase(up_version)?;
    testcase(up_incorrect_crc32)?;
    testcase(up_crc32_source)?;
    testcase(up_invalid)?;
    testcase(up_efs)?;
    Ok(())
}
