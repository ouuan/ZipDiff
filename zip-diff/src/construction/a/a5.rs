use crate::utils::testcase;
use anyhow::Result;
use zip_diff::fields::GeneralPurposeFlag;
use zip_diff::zip::ZipArchive;

fn lfh_enc() -> Result<ZipArchive> {
    let mut zip = ZipArchive::default();

    zip.add_simple("test", b"test")?;
    zip.finalize()?;
    zip.files[0]
        .lfh
        .general_purpose_flag
        .insert(GeneralPurposeFlag::Encrypted);

    Ok(zip)
}

fn cdh_enc() -> Result<ZipArchive> {
    let mut zip = ZipArchive::default();

    zip.add_simple("test", b"test")?;
    zip.finalize()?;
    zip.cd[0]
        .general_purpose_flag
        .insert(GeneralPurposeFlag::Encrypted);

    Ok(zip)
}

fn first_enc() -> Result<ZipArchive> {
    let mut zip = ZipArchive::default();

    zip.add_simple("first", b"first")?;
    zip.add_simple("second", b"second")?;
    zip.files[0]
        .lfh
        .general_purpose_flag
        .insert(GeneralPurposeFlag::Encrypted);
    zip.finalize()?;

    Ok(zip)
}

pub fn main() -> Result<()> {
    testcase(lfh_enc)?;
    testcase(cdh_enc)?;
    testcase(first_enc)?;
    Ok(())
}
