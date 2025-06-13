use crate::utils::testcase;
use anyhow::Result;
use zip_diff::fields::CompressionMethod;
use zip_diff::zip::ZipArchive;

const DATA: &[u8] = b"test";

fn stored_lfh() -> Result<ZipArchive> {
    let mut zip = ZipArchive::default();
    zip.add_file("test", DATA, CompressionMethod::DEFLATED, false, false)?;
    zip.finalize()?;
    zip.files[0].lfh.compression_method = CompressionMethod::STORED;
    zip.files[0].lfh.compressed_size = DATA.len().try_into().unwrap();
    Ok(zip)
}

fn stored_cdh() -> Result<ZipArchive> {
    let mut zip = ZipArchive::default();
    zip.add_file("test", DATA, CompressionMethod::DEFLATED, false, false)?;
    zip.finalize()?;
    zip.cd[0].compression_method = CompressionMethod::STORED;
    zip.cd[0].compressed_size = DATA.len().try_into().unwrap();
    Ok(zip)
}

pub fn main() -> Result<()> {
    testcase(stored_lfh)?;
    testcase(stored_cdh)?;
    Ok(())
}
