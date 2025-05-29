use crate::cdh::CentralDirectoryHeader;
use crate::compress::decompress;
use crate::fields::CompressionMethod;
use crate::zip::FileEntry;
use anyhow::{anyhow, Context, Result};
use binwrite::{BinWrite, WriterOption};
use std::fmt::{self, Debug, Formatter};
use std::io::{self, Write};

pub fn binwrite_option<W, T>(
    option: &Option<T>,
    writer: &mut W,
    options: &WriterOption,
) -> io::Result<()>
where
    W: Write,
    T: BinWrite,
{
    if let Some(val) = option {
        val.write_options(writer, options)?;
    }
    Ok(())
}

pub trait BinWriteTransform {
    type Type: BinWrite;

    fn binwrite_transform(&self) -> io::Result<Self::Type>;
}

pub fn binwrite_transform<W, T>(var: &T, writer: &mut W, options: &WriterOption) -> io::Result<()>
where
    W: Write,
    T: BinWriteTransform,
{
    var.binwrite_transform()?.write_options(writer, options)
}

pub trait BinCount {
    /// Count how many bytes would be written via `BinWrite`.
    fn byte_count(&self) -> Result<usize>;
}

impl<T: BinWrite + ?Sized> BinCount for T {
    fn byte_count(&self) -> Result<usize> {
        let mut counter = WriteCounter::new();
        self.write(&mut counter)?;
        Ok(counter.count)
    }
}

struct WriteCounter {
    count: usize,
}

impl WriteCounter {
    fn new() -> Self {
        WriteCounter { count: 0 }
    }
}

impl Write for WriteCounter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.count += buf.len();
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub fn find_file<T, I>(iter: I, file_name: &str) -> Result<(usize, T)>
where
    T: GetFileName,
    I: IntoIterator<Item = T>,
{
    iter.into_iter()
        .enumerate()
        .find(|(_, f)| f.get_file_name() == file_name.as_bytes())
        .context(format!("Failed to find {}", file_name))
}

pub trait GetFileName {
    fn get_file_name(&self) -> &[u8];
}

impl GetFileName for FileEntry {
    fn get_file_name(&self) -> &[u8] {
        &self.lfh.file_name
    }
}

impl GetFileName for &FileEntry {
    fn get_file_name(&self) -> &[u8] {
        &self.lfh.file_name
    }
}

impl GetFileName for &mut FileEntry {
    fn get_file_name(&self) -> &[u8] {
        &self.lfh.file_name
    }
}

impl GetFileName for &mut CentralDirectoryHeader {
    fn get_file_name(&self) -> &[u8] {
        &self.file_name
    }
}

pub fn align_entry_size(entries: &mut [&mut FileEntry], padding: u8) -> Result<()> {
    for entry in entries.iter_mut() {
        entry.data = decompress(entry.lfh.compression_method, &entry.data)?;
    }

    let max_len = entries
        .iter()
        .map(|entry| entry.data.len())
        .max()
        .ok_or(anyhow!("no entry provided"))?;

    for entry in entries {
        entry.data.resize(max_len, padding);
        entry.lfh.compressed_size = max_len as u32;
        entry.lfh.uncompressed_size = max_len as u32;
        entry.lfh.compression_method = CompressionMethod::STORED;
        entry.lfh.crc32 = crc32fast::hash(&entry.data);
    }

    Ok(())
}

pub fn fmt_utf8(b: &[u8], f: &mut Formatter) -> fmt::Result {
    std::str::from_utf8(b).map_err(|_| fmt::Error)?.fmt(f)
}

pub fn fmt_hex(b: &[u8], f: &mut Formatter) -> fmt::Result {
    for x in b {
        write!(f, "{x:02x} ")?;
    }
    Ok(())
}

// reference: https://github.com/shuax/custom_crc32
pub fn crc32_patch(data: &[u8], target: u32) -> u32 {
    const fn crc32_rev(byte: u32) -> u32 {
        const POLY: u32 = 0xedb88320;
        let mut x = byte << 24;
        let mut i = 0;
        while i < 8 {
            if x & 0x80000000 != 0 {
                x = ((x ^ POLY) << 1) | 1;
            } else {
                x <<= 1;
            }
            i += 1;
        }
        x
    }

    let current = !crc32fast::hash(data);
    let mut result = !target;
    for i in 0..4 {
        result = (result << 8) ^ crc32_rev(result >> 24) ^ ((current >> ((3 - i) * 8)) & 0xff);
    }
    result
}
