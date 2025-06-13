use crate::utils::{testcase, testcase_arg};
use anyhow::Result;
use bitflags::bitflags;
use zip_diff::utils::BinCount;
use zip_diff::zip::ZipArchive;

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

    if flags.contains(CdhCountFlags::ThisDiskCount) {
        zip.eocdr.this_disk_cdh_count -= 1;
    }

    if flags.contains(CdhCountFlags::TotalCount) {
        zip.eocdr.total_cdh_count -= 1;
    }

    if flags.contains(CdhCountFlags::CdSize) {
        zip.eocdr.size_of_cd = zip.cd[0].byte_count()?.try_into()?;
    }

    Ok(zip)
}

fn modulo_65536() -> Result<ZipArchive> {
    let mut zip = ZipArchive::default();

    for i in 1u32..=65537 {
        zip.add_simple(&format!("{:x}/{:x}", i / 256, i % 256), b"")?;
    }

    zip.finalize()?;

    let zip64_eocdr = zip.zip64_eocdr.as_mut().unwrap();
    zip64_eocdr.this_disk_cdh_count -= 65536;
    zip64_eocdr.total_cdh_count -= 65536;
    zip.eocdr = (&*zip64_eocdr).try_into()?;
    zip.zip64_eocdr = None;
    zip.zip64_eocdl = None;

    Ok(zip)
}

pub fn main() -> Result<()> {
    (1..8).try_for_each(|flags| testcase_arg(cdh_count, CdhCountFlags::from_bits_truncate(flags)))?;
    testcase(modulo_65536)?;
    Ok(())
}
