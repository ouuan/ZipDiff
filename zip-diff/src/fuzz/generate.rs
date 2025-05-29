use crate::config::CONFIG;
use rand::distributions::{DistString, Standard};
use rand::prelude::*;
use zip_diff::fields::CompressionMethod;
use zip_diff::zip::ZipArchive;

pub fn init_corpus() -> Vec<ZipArchive> {
    let mut result = Vec::new();

    let mut zip = ZipArchive::default();
    zip.add_simple("a", b"a").unwrap();
    zip.add_simple("b/c", b"c").unwrap();
    zip.add_simple("b/d", b"d").unwrap();
    zip.finalize().unwrap();
    result.push(zip);

    let mut rng = thread_rng();

    for _ in 0..CONFIG.batch_size {
        let mut zip = ZipArchive::default();
        let count = rng.gen_range(0..5);
        for _ in 0..count {
            let name_len = rng.gen_range(0..5);
            let name = Standard.sample_string(&mut rng, name_len);
            let data_len = rng.gen_range(0..10);
            let mut data = Vec::with_capacity(data_len);
            data.resize_with(data_len, || rng.gen());
            let compression = match rng.gen_range(0..16) {
                0..8 => CompressionMethod::STORED,
                8..12 => CompressionMethod::DEFLATED,
                12 => CompressionMethod::BZIP2,
                13 => CompressionMethod::ZSTD,
                14 => CompressionMethod::LZMA,
                15 => CompressionMethod::XZ,
                _ => unreachable!(),
            };
            zip.add_file(
                &name,
                &data,
                compression,
                rng.gen_ratio(1, 5),
                rng.gen_ratio(1, 5),
            )
            .unwrap();
        }
        if rng.gen_ratio(1, 5) {
            zip.set_eocd(true).unwrap();
        }
        zip.finalize().unwrap();
        result.push(zip);
    }

    result
}
