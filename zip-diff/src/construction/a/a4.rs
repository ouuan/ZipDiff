use crate::utils::{testcase, testcase_arg};
use anyhow::Result;
use zip_diff::zip::ZipArchive;

const DOS_ATTR: u32 = 0x10;
const UNIX_ATTR: u32 = 0x4000 << 16;
const DOS_VER: u16 = 0;
const UNIX_VER: u16 = 3 << 8;
const OSX_VER: u16 = 19 << 8;

fn slash() -> Result<ZipArchive> {
    let mut zip = ZipArchive::default();
    zip.add_simple("test/", b"test")?;
    zip.finalize()?;
    Ok(zip)
}

fn backslash() -> Result<ZipArchive> {
    let mut zip = ZipArchive::default();
    zip.add_simple("test\\", b"test")?;
    zip.finalize()?;
    Ok(zip)
}

fn slash_empty() -> Result<ZipArchive> {
    let mut zip = ZipArchive::default();
    zip.add_simple("test/", b"")?;
    zip.finalize()?;
    Ok(zip)
}

fn external_attr(arg: u8) -> Result<ZipArchive> {
    let mut zip = ZipArchive::default();
    zip.add_simple("test", b"test")?;
    zip.finalize()?;
    zip.cd[0].external_file_attributes |= if arg / 3 == 0 { DOS_ATTR } else { UNIX_ATTR };
    zip.cd[0].version_made_by |= match arg % 3 {
        0 => DOS_VER,
        1 => UNIX_VER,
        2 => OSX_VER,
        _ => unreachable!(),
    };
    Ok(zip)
}

pub fn main() -> Result<()> {
    testcase(slash)?;
    testcase(backslash)?;
    testcase(slash_empty)?;
    (0..6).try_for_each(|arg| testcase_arg(external_attr, arg))?;
    Ok(())
}
