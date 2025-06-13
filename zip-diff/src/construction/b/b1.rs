use crate::utils::testcase;
use anyhow::Result;
use zip_diff::zip::ZipArchive;

fn duplicate() -> Result<ZipArchive> {
    let mut zip = ZipArchive::default();
    zip.add_simple("test", b"a")?;
    zip.add_simple("test", b"b")?;
    zip.finalize()?;
    Ok(zip)
}

pub fn main() -> Result<()> {
    testcase(duplicate)?;
    Ok(())
}
