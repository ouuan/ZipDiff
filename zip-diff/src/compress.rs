use crate::fields::CompressionMethod;
use anyhow::{bail, Context, Result};
use bzip2::{bufread::BzDecoder, write::BzEncoder, Compression as BzCompression};
use flate2::{bufread::DeflateDecoder, write::DeflateEncoder, Compression as DeflateCompression};
use lzma_rs::{lzma_compress, lzma_decompress, xz_compress, xz_decompress};
use std::io::{Cursor, Read, Write};

pub fn compress(method: CompressionMethod, data: &[u8]) -> Result<Vec<u8>> {
    match method {
        CompressionMethod::STORED => Ok(Vec::from(data)),
        CompressionMethod::DEFLATED => {
            let mut encoder = DeflateEncoder::new(Vec::new(), DeflateCompression::default());
            encoder.write_all(data).context("Failed to deflate")?;
            encoder.finish().context("Failed to deflate")
        }
        CompressionMethod::BZIP2 => {
            let mut encoder = BzEncoder::new(Vec::new(), BzCompression::default());
            encoder.write_all(data).context("Failed to bzip2")?;
            encoder.finish().context("Failed to bzip2")
        }
        CompressionMethod::ZSTD => zstd::encode_all(data, 0).context("Failed to ZSTD compress"),
        CompressionMethod::LZMA => {
            let mut input = Cursor::new(data);
            let mut output = Vec::new();
            lzma_compress(&mut input, &mut output).context("Failed to LZMA compress")?;
            Ok(output)
        }
        CompressionMethod::XZ => {
            let mut input = Cursor::new(data);
            let mut output = Vec::new();
            xz_compress(&mut input, &mut output).context("Failed to XZ compress")?;
            Ok(output)
        }
        _ => bail!("Compression method {:?} not implemented", method),
    }
}

pub fn decompress(method: CompressionMethod, data: &[u8]) -> Result<Vec<u8>> {
    match method {
        CompressionMethod::STORED => Ok(Vec::from(data)),
        CompressionMethod::DEFLATED => {
            let mut decoder = DeflateDecoder::new(data);
            let mut buf = Vec::new();
            decoder.read_to_end(&mut buf).context("Failed to inflate")?;
            Ok(buf)
        }
        CompressionMethod::BZIP2 => {
            let mut decoder = BzDecoder::new(data);
            let mut buf = Vec::new();
            decoder.read_to_end(&mut buf).context("Failed to bunzip2")?;
            Ok(buf)
        }
        CompressionMethod::ZSTD => zstd::decode_all(data).context("Failed to ZSTD decompress"),
        CompressionMethod::LZMA => {
            let mut input = Cursor::new(data);
            let mut output = Vec::new();
            lzma_decompress(&mut input, &mut output).context("Failed to LZMA decompress")?;
            Ok(output)
        }
        CompressionMethod::XZ => {
            let mut input = Cursor::new(data);
            let mut output = Vec::new();
            xz_decompress(&mut input, &mut output).context("Failed to XZ decompress")?;
            Ok(output)
        }
        _ => bail!("Decompression method {:?} not implemented", method),
    }
}
