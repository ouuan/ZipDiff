use crate::utils::testcase;
use anyhow::Result;
use zip_diff::zip::ZipArchive;

fn casing() -> Result<ZipArchive> {
    let mut zip = ZipArchive::default();
    zip.add_simple("test.txt", b"a")?;
    zip.add_simple("test.TXT", b"b")?;
    zip.finalize()?;
    Ok(zip)
}

pub fn main() -> Result<()> {
    testcase(casing)?;
    Ok(())
}
