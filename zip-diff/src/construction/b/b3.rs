use crate::utils::testcase_arg;
use anyhow::Result;
use zip_diff::zip::ZipArchive;

fn canonical_first(path: &str) -> Result<ZipArchive> {
    let mut zip = ZipArchive::default();
    zip.add_simple("a/b", b"a")?;
    zip.add_simple(path, b"b")?;
    zip.finalize()?;
    Ok(zip)
}

fn canonical_second(path: &str) -> Result<ZipArchive> {
    let mut zip = ZipArchive::default();
    zip.add_simple(path, b"a")?;
    zip.add_simple("a/b", b"b")?;
    zip.finalize()?;
    Ok(zip)
}

pub fn main() -> Result<()> {
    [
        "/a/b",
        "a//b",
        "a\\b",
        "./a/b",
        "a/./b",
        "a/b/.",
        "../a/b",
        ".../a/b",
        "a/.../b",
        "c/../a/b",
    ]
    .into_iter()
    .try_for_each(|path| {
        testcase_arg(canonical_first, path)?;
        testcase_arg(canonical_second, path)
    })?;
    Ok(())
}
