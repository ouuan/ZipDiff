use anyhow::{Context, Result};
use binwrite::BinWrite;
use std::any::type_name_of_val;
use std::collections::BTreeMap;
use std::fs::{create_dir_all, File};
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::sync::Mutex;
use zip_diff::cdh::CentralDirectoryHeader;
use zip_diff::zip::{FileEntry, ZipArchive};

static WRITE_COUNTER: Mutex<BTreeMap<String, usize>> = Mutex::new(BTreeMap::new());

fn write_core<Z: BinWrite>(ambiguity_type: &str, data: Z) -> Result<()> {
    let count = *WRITE_COUNTER
        .lock()
        .unwrap()
        .entry(ambiguity_type.to_string())
        .and_modify(|e| *e += 1)
        .or_insert(1);
    let path = format!("../constructions/{ambiguity_type}/{ambiguity_type}-{count}.zip");
    let path = PathBuf::from(path);
    create_dir_all(path.parent().unwrap())?;
    let file = File::create(path).context("failed to create sample file")?;
    let mut writer = BufWriter::new(file);
    data.write(&mut writer)
        .context("failed to write sample file")?;
    writer.flush().context("failed to flush sample file writer")
}

pub fn testcase<Z, F>(construction: F) -> Result<()>
where
    Z: BinWrite,
    F: FnOnce() -> Result<Z>,
{
    let ambiguity_type = type_name_of_val(&construction).rsplit("::").nth(1).unwrap();
    let data = construction()?;
    write_core(ambiguity_type, data)
}

pub fn testcase_arg<Z, A, F>(construction: F, arg: A) -> Result<()>
where
    Z: BinWrite,
    F: FnOnce(A) -> Result<Z>,
{
    let ambiguity_type = type_name_of_val(&construction).rsplit("::").nth(1).unwrap();
    let data = construction(arg)?;
    write_core(ambiguity_type, data)
}

#[derive(BinWrite)]
pub struct EntryGroup {
    pub files: Vec<FileEntry>,
    pub cd: Vec<CentralDirectoryHeader>,
}

impl From<ZipArchive> for EntryGroup {
    fn from(zip: ZipArchive) -> Self {
        Self {
            files: zip.files,
            cd: zip.cd,
        }
    }
}

// Two strings with the same length and CRC32
// https://www.thecodingforums.com/threads/finding-two-strings-with-the-same-crc32.889011/#post-4775592
pub const CRC32A: &str = "oxueekz";
pub const CRC32B: &str = "pyqptgs";
