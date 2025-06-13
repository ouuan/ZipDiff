use crate::utils::testcase_arg;
use anyhow::Result;
use zip_diff::zip::ZipArchive;

enum Host {
    Dos,
    Unix,
    Both,
}

const UNIX_VER: u16 = 3 << 8;

fn special_byte((byte, host): (u8, Host)) -> Result<ZipArchive> {
    let mut zip = ZipArchive::default();
    zip.add_simple("a b", b"")?;
    zip.files[0].lfh.file_name[1] = byte;
    if matches!(host, Host::Both) {
        zip.files.push(zip.files[0].clone());
    }
    zip.finalize()?;
    if matches!(host, Host::Unix | Host::Both) {
        zip.cd[0].version_made_by |= UNIX_VER;
    }
    Ok(zip)
}

fn two_special_bytes((a, b): (u8, u8)) -> Result<ZipArchive> {
    let mut zip = ZipArchive::default();
    zip.add_simple("a b", b"")?;
    zip.add_simple("a b", b"")?;
    zip.files[0].lfh.file_name[1] = a;
    zip.files[1].lfh.file_name[1] = b;
    zip.finalize()?;
    Ok(zip)
}

pub fn main() -> Result<()> {
    for byte in 0..=u8::MAX {
        if byte.is_ascii_alphanumeric() {
            continue;
        }
        for host in [Host::Dos, Host::Unix, Host::Both] {
            testcase_arg(special_byte, (byte, host))?;
        }
    }
    for a in (0..=u8::MAX)
        .step_by(8)
        .filter(|&x| !x.is_ascii_alphanumeric())
    {
        for b in (7..=u8::MAX)
            .step_by(8)
            .filter(|&x| !x.is_ascii_alphanumeric())
        {
            testcase_arg(two_special_bytes, (a, b))?;
        }
    }
    Ok(())
}
