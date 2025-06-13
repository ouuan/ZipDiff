use crate::rand_utils::*;
use crate::Input;
use binwrite::BinWrite;
use blake3::Hash;
use itertools::izip;
use num_traits::{bounds::UpperBounded, NumCast, Saturating, Unsigned, Zero};
use rand::distributions::{DistString, Standard};
use rand::prelude::*;
use serde::Serialize;
use std::any::type_name;
use std::borrow::Cow;
use std::collections::BTreeMap;
use vec_box::vec_box;
use zip_diff::cdh::CentralDirectoryHeader;
use zip_diff::compress::*;
use zip_diff::dd::{DataDescriptor, U32or64};
use zip_diff::eocd::*;
use zip_diff::extra::*;
use zip_diff::fields::*;
use zip_diff::utils::BinCount;
use zip_diff::zip::ZipArchive;

pub struct Mutator {
    zip_mutations: Vec<Box<dyn ZipMutate + Sync>>,
    zip_ucb: Ucb,
    bytes_mutations: Vec<Box<dyn BytesMutate + Sync>>,
    bytes_ucb: Ucb,
}

pub struct UcbHandle {
    zip_id: Option<usize>,
    bytes_id: Option<usize>,
}

#[derive(Serialize)]
pub struct MutationStats {
    zip: BTreeMap<&'static str, (f64, f64)>,
    bytes: BTreeMap<&'static str, (f64, f64)>,
}

pub struct Sample {
    pub input: Input,
    pub bytes: Vec<u8>,
    pub mutations: Vec<&'static str>,
    pub hash: Hash,
    pub name: String,
    pub ucb_handles: Vec<UcbHandle>,
}

impl Mutator {
    pub fn new() -> Self {
        let zip_mutations: Vec<Box<dyn ZipMutate + Sync>> = vec_box![
            FixZip,
            SetOffsets,
            AddFileEntry,
            RemoveLfh,
            RemoveCdh,
            ModifyVersionNeeded,
            FlipGeneralPurposeFlag,
            ModifyCompressionMethod,
            ModifyLastMod,
            ModifyCrc32,
            ModifyCompressedSize,
            ModifyUncompressedSize,
            ModifyFileNameLength,
            ModifyFileName,
            ModifyFileNameAndLength,
            ModifyFileNameCasing,
            AddPathCharInName,
            ModifyExtraFieldLength,
            AddZip64ExtraField,
            AddUpExtraField,
            RemoveExtraField,
            AddDataDescriptor,
            ModifyCdhVersionMadeBy,
            ModifyCdhComment,
            ModifyCdhCommentLength,
            ModifyCdhCommentAndLength,
            ModifyCdhDiskNumberStart,
            FlipCdhInternalFileAttributes,
            FlipCdhExternalFileAttributes,
            ModifyCdhRelativeHeaderOffset,
            ModifyContentCompression,
            ModifyContentSize,
            ModifyEocdrCurrentDisk,
            ModifyEocdrStartOfCdDisk,
            ModifyEocdrThisDiskCdhCount,
            ModifyEocdrTotalCdhCount,
            ModifyEocdrCdSize,
            ModifyEocdrCdOffset,
            ModifyEocdrComment,
            ModifyEocdrCommentLength,
            ModifyEocdrCommentAndLength,
            UseZip64Eocd,
            UseZip64EocdNoFf,
            ModifyZip64Eocdr,
            UseZip64EocdrV2,
            ModifyEocdl,
        ];
        let bytes_mutations: Vec<Box<dyn BytesMutate + Sync>> = vec_box![
            ModifyByte,
            FlipBit,
            InsertBytes,
            DeleteBytes,
            DuplicateBytes,
            SpliceBytes,
        ];
        let zip_ucb = Ucb::new(zip_mutations.len() + 1); // ZIP can use bytes mutation
        let bytes_ucb = Ucb::new(bytes_mutations.len());
        Self {
            zip_mutations,
            zip_ucb,
            bytes_mutations,
            bytes_ucb,
        }
    }

    pub fn construct_ucb(&mut self) {
        self.zip_ucb.construct();
        self.bytes_ucb.construct();
    }

    pub fn record_ucb(&mut self, results: &[(Vec<UcbHandle>, bool)]) {
        for (handles, success) in results {
            let trial = 1.0 / handles.len() as f64;
            let score = if *success { trial } else { 0.0 };
            for handle in handles {
                if let Some(id) = handle.zip_id {
                    self.zip_ucb.record(id, trial, score);
                }
                if let Some(id) = handle.bytes_id {
                    self.bytes_ucb.record(id, trial, score);
                }
            }
        }
    }

    pub fn stats(&self) -> MutationStats {
        MutationStats {
            zip: izip!(
                self.zip_mutations
                    .iter()
                    .map(|m| m.name())
                    .chain(std::iter::once("BytesMutate")),
                self.zip_ucb.scores(),
                self.zip_ucb.trials()
            )
            .map(|(m, &s, &t)| (m, (s / t, t)))
            .collect(),
            bytes: izip!(
                &self.bytes_mutations,
                self.bytes_ucb.scores(),
                self.bytes_ucb.trials()
            )
            .map(|(m, &s, &t)| (m.name(), (s / t, t)))
            .collect(),
        }
    }

    pub fn mutate(&self, input: &Input, rng: &mut ThreadRng) -> (Input, &'static str, UcbHandle) {
        let (bytes, zip_id) = match input {
            Input::Zip(zip) => {
                loop {
                    let p = self.zip_ucb.sample(rng);
                    if let Some(mutation) = self.zip_mutations.get(p) {
                        let mut zip = zip.clone();
                        if mutation.mutate(&mut zip, rng).is_some() {
                            return (
                                Input::Zip(zip),
                                mutation.name(),
                                UcbHandle {
                                    zip_id: Some(p),
                                    bytes_id: None,
                                },
                            );
                        }
                    } else {
                        // use bytes mutation
                        let mut buf = Vec::new();
                        zip.write(&mut buf).unwrap();
                        break (Cow::Owned(buf), Some(self.zip_mutations.len()));
                    }
                }
            }
            Input::Bytes(bytes) => (Cow::Borrowed(bytes), None),
        };
        // try byte mutation until success
        loop {
            let p = self.bytes_ucb.sample(rng);
            let mutation = &self.bytes_mutations[p];
            let mut bytes = bytes.as_ref().to_vec();
            if mutation.mutate(&mut bytes, rng).is_some() {
                return (
                    Input::Bytes(bytes),
                    mutation.name(),
                    UcbHandle {
                        zip_id,
                        bytes_id: Some(p),
                    },
                );
            }
        }
    }

    pub fn generate_sample(
        &self,
        input: &Input,
        mutations: &[&'static str],
        mutate_times: usize,
        rng: &mut ThreadRng,
    ) -> Sample {
        let mut input = Cow::Borrowed(input);
        let mut mutations = mutations.to_vec();
        let mut ucb_handles = Vec::new();
        for _ in 0..mutate_times {
            let (new_input, mutation, ucb_handle) = self.mutate(input.as_ref(), rng);
            input = Cow::Owned(new_input);
            mutations.push(mutation);
            ucb_handles.push(ucb_handle);
        }
        let input = input.into_owned();
        let bytes = match &input {
            Input::Zip(zip) => {
                let mut buf = Vec::new();
                zip.write(&mut buf).expect("failed to write ZIP file");
                buf
            }
            Input::Bytes(bytes) => bytes.clone(),
        };
        let hash = blake3::hash(&bytes);
        let name = format!("{}.zip", hash.to_hex());
        Sample {
            input,
            bytes,
            hash,
            name,
            mutations,
            ucb_handles,
        }
    }
}

fn name_of<T: ?Sized>() -> &'static str {
    let type_name = type_name::<T>();
    type_name.rsplit("::").next().unwrap_or(type_name)
}

trait ZipMutate {
    /// Mutate the *zip*. Return `Some(())` on success, return `None` on failure.
    fn mutate(&self, zip: &mut ZipArchive, rng: &mut ThreadRng) -> Option<()>;

    fn name(&self) -> &'static str {
        name_of::<Self>()
    }
}

trait BytesMutate {
    /// Mutate the *bytes*. Return `Some(())` on success, return `None` on failure.
    fn mutate(&self, bytes: &mut Vec<u8>, rng: &mut ThreadRng) -> Option<()>;

    fn name(&self) -> &'static str {
        name_of::<Self>()
    }
}

struct FixZip;

impl ZipMutate for FixZip {
    fn mutate(&self, zip: &mut ZipArchive, _: &mut ThreadRng) -> Option<()> {
        zip.finalize().ok()
    }
}

struct SetOffsets;

impl ZipMutate for SetOffsets {
    fn mutate(&self, zip: &mut ZipArchive, rng: &mut ThreadRng) -> Option<()> {
        let base = rand_len(rng) - 1;
        zip.set_offsets(base).ok()
    }
}

struct AddFileEntry;

impl ZipMutate for AddFileEntry {
    fn mutate(&self, zip: &mut ZipArchive, rng: &mut ThreadRng) -> Option<()> {
        let name_len = rand_len(rng);
        let name = Standard.sample_string(rng, name_len);
        let mut data = vec![0; rand_len(rng)];
        rng.fill_bytes(&mut data[..]);
        let compression_method = if rng.gen() {
            CompressionMethod::DEFLATED
        } else {
            CompressionMethod::STORED
        };
        let force_zip64 = rng.gen();
        let use_dd = rng.gen();
        zip.add_file(&name, &data, compression_method, force_zip64, use_dd)
            .ok()?;
        zip.finalize().ok()
    }
}

struct RemoveLfh;

impl ZipMutate for RemoveLfh {
    fn mutate(&self, zip: &mut ZipArchive, rng: &mut ThreadRng) -> Option<()> {
        let index = (0..zip.files.len()).choose(rng)?;
        zip.files.remove(index);
        Some(())
    }
}

struct RemoveCdh;

impl ZipMutate for RemoveCdh {
    fn mutate(&self, zip: &mut ZipArchive, rng: &mut ThreadRng) -> Option<()> {
        let index = (0..zip.cd.len()).choose(rng)?;
        zip.cd.remove(index);
        Some(())
    }
}

macro_rules! entry_field_mutation {
    ($struct_name:ident, [$($field_name:ident),+], $rng:ident, $header:ident, $mutation:stmt) => {
        struct $struct_name;

        impl ZipMutate for $struct_name {
            fn mutate(&self, zip: &mut ZipArchive, $rng: &mut ThreadRng) -> Option<()> {
                let (index, loc) = rand_header(zip, $rng)?;

                if loc.lfh() {
                    let $header = &mut zip.files[index].lfh;
                    $(let $field_name = &mut $header.$field_name;)+
                    $mutation
                }

                if loc.cdh() {
                    let $header = &mut zip.cd[index];
                    $(let $field_name = &mut $header.$field_name;)+
                    $mutation
                }

                Some(())
            }
        }
    };
    ($struct_name:ident, [$($field_name:ident),+], $rng:ident, $mutation:stmt) => {
        entry_field_mutation!($struct_name, [$($field_name),+], $rng, header, $mutation);
    };
    ($struct_name:ident, $field_name:ident, $rng:ident, $mutation:stmt) => {
        entry_field_mutation!($struct_name, [$field_name], $rng, $mutation);
    };
}

entry_field_mutation!(ModifyVersionNeeded, version_needed, rng, {
    *version_needed = rng.gen();
});

entry_field_mutation!(FlipGeneralPurposeFlag, general_purpose_flag, rng, {
    let bit = rng.gen_range(0..16);
    general_purpose_flag.toggle(GeneralPurposeFlag::from_bits_retain(1 << bit));
});

fn rand_compression(current: CompressionMethod, rng: &mut ThreadRng) -> CompressionMethod {
    loop {
        let new = match rng.gen_range(0..20) {
            0 => CompressionMethod(rng.gen()),
            1 => CompressionMethod::BZIP2,
            2 => CompressionMethod::XZ,
            3 => CompressionMethod::LZMA,
            4 => CompressionMethod::ZSTD,
            5..10 => CompressionMethod::DEFLATED,
            _ => CompressionMethod::STORED,
        };
        if new != current {
            return new;
        }
    }
}

entry_field_mutation!(
    ModifyCompressionMethod,
    compression_method,
    rng,
    *compression_method = rand_compression(*compression_method, rng)
);

entry_field_mutation!(ModifyLastMod, last_mod, rng, {
    if rng.gen() {
        last_mod.time = rng.gen();
    } else {
        last_mod.date = rng.gen();
    }
});

entry_field_mutation!(ModifyCrc32, crc32, rng, {
    if rng.gen() {
        *crc32 = 0;
    } else {
        *crc32 = rng.gen();
    }
});

entry_field_mutation!(ModifyCompressedSize, compressed_size, rng, {
    match rng.gen_range(0..50) {
        0 => *compressed_size = u32::MAX,
        1..10 => *compressed_size = 0,
        _ => mutate_len(compressed_size, rng),
    }
});

entry_field_mutation!(ModifyUncompressedSize, uncompressed_size, rng, {
    match rng.gen_range(0..50) {
        0 => *uncompressed_size = u32::MAX,
        1..10 => *uncompressed_size = 0,
        _ => mutate_len(uncompressed_size, rng),
    }
});

entry_field_mutation!(ModifyFileNameLength, file_name_length, rng, {
    mutate_len(file_name_length, rng);
});

entry_field_mutation!(ModifyFileName, file_name, rng, {
    *file_name.choose_mut(rng)? = rng.gen();
});

entry_field_mutation!(
    ModifyFileNameAndLength,
    [file_name, file_name_length],
    rng,
    {
        mutate_len(file_name_length, rng);
        file_name.resize_with(*file_name_length as usize, || rng.gen());
    }
);

entry_field_mutation!(ModifyFileNameCasing, file_name, rng, {
    file_name.iter_mut().for_each(|c| {
        if c.is_ascii_lowercase() {
            c.make_ascii_uppercase();
        } else {
            c.make_ascii_lowercase();
        }
    });
});

entry_field_mutation!(AddPathCharInName, [file_name, file_name_length], rng, {
    let len = rand_len(rng);
    *file_name_length += len as u16;
    for _ in 0..len {
        let index = rng.gen_range(0..=file_name.len());
        file_name.insert(index, *br"./\".choose(rng).unwrap());
    }
});

entry_field_mutation!(
    ModifyExtraFieldLength,
    extra_field_length,
    rng,
    mutate_len(extra_field_length, rng)
);

entry_field_mutation!(
    AddZip64ExtraField,
    [compressed_size, uncompressed_size, extra_fields],
    rng,
    header,
    {
        let original_size = match rng.gen_range(0..3) {
            0 => None,
            1 => Some(*uncompressed_size as _),
            _ => Some(0),
        };
        let zip64_compressed_size = match rng.gen_range(0..3) {
            0 => None,
            1 => Some(*compressed_size as _),
            _ => Some(0),
        };

        let extra = Zip64ExtendedInfo {
            original_size,
            compressed_size: zip64_compressed_size,
            ..Default::default()
        };
        extra_fields.push(extra.into());

        if rng.gen() {
            *compressed_size = u32::MAX;
        }
        if rng.gen() {
            *uncompressed_size = u32::MAX;
        }

        header.finalize().unwrap();
    }
);

entry_field_mutation!(AddUpExtraField, [file_name, extra_fields], rng, header, {
    let version = std::iter::repeat_n(1, 5)
        .chain([0, 2, u8::MAX])
        .choose(rng)
        .unwrap();

    let unicode_name_len = rng.gen_range(0..10);
    let unicode_name = Standard.sample_string(rng, unicode_name_len);

    let crc_name = match rng.gen_range(0..4) {
        0 => extra_fields
            .iter()
            .filter_map(|f| {
                Some(
                    f.data
                        .downcast_ref::<InfoZipUnicodePath>()?
                        .unicode_name
                        .as_bytes(),
                )
            })
            .choose(rng)
            .unwrap_or(file_name),
        1 => unicode_name.as_bytes(),
        _ => file_name,
    };
    let name_crc32 = crc32fast::hash(crc_name);

    let up = InfoZipUnicodePath {
        version,
        name_crc32,
        unicode_name,
    };
    extra_fields.push(up.into());

    header.finalize().unwrap();
});

entry_field_mutation!(RemoveExtraField, [extra_fields], rng, header, {
    let len = extra_fields.len();
    if len > 0 {
        let index = rng.gen_range(0..len);
        extra_fields.remove(index);
    }
});

struct AddDataDescriptor;

impl ZipMutate for AddDataDescriptor {
    fn mutate(&self, zip: &mut ZipArchive, rng: &mut ThreadRng) -> Option<()> {
        let entry = &mut zip.files.iter_mut().choose(rng)?;
        entry
            .lfh
            .general_purpose_flag
            .insert(GeneralPurposeFlag::DataDescriptor);

        if let Some(dd) = &entry.dd {
            if entry.lfh.crc32 == 0 {
                entry.lfh.crc32 = dd.crc32;
            } else {
                entry.lfh.crc32 = 0;
            }
            if entry.lfh.compressed_size == 0 {
                entry.lfh.compressed_size = dd.compressed_size.saturate();
            } else {
                entry.lfh.compressed_size = 0;
            }
            if entry.lfh.uncompressed_size == 0 {
                entry.lfh.uncompressed_size = dd.uncompressed_size.saturate();
            } else {
                entry.lfh.uncompressed_size = 0;
            }
        } else {
            let signature = if rng.gen() {
                Some(DataDescriptor::SIGNATURE)
            } else {
                None
            };
            let crc32 = if rng.gen() { entry.lfh.crc32 } else { 0 };
            let compressed_size = if rng.gen() {
                entry.lfh.compressed_size
            } else {
                0
            };
            let uncompressed_size = if rng.gen() {
                entry.lfh.uncompressed_size
            } else {
                0
            };
            let (compressed_size, uncompressed_size) = if rng.gen_ratio(1, 10) {
                (
                    U32or64::U64(compressed_size as _),
                    U32or64::U64(uncompressed_size as _),
                )
            } else {
                (
                    U32or64::U32(compressed_size),
                    U32or64::U32(uncompressed_size),
                )
            };
            entry.dd = Some(DataDescriptor {
                signature,
                crc32,
                compressed_size,
                uncompressed_size,
            });

            if rng.gen() {
                entry.lfh.crc32 = 0;
                entry.lfh.compressed_size = 0;
                entry.lfh.uncompressed_size = 0;
            } else {
                if rng.gen() {
                    entry.lfh.crc32 = 0;
                }
                if rng.gen() {
                    entry.lfh.compressed_size = 0;
                }
                if rng.gen() {
                    entry.lfh.uncompressed_size = 0;
                }
            }
        }

        Some(())
    }
}

struct ModifyCdhVersionMadeBy;

impl ZipMutate for ModifyCdhVersionMadeBy {
    fn mutate(&self, zip: &mut ZipArchive, rng: &mut ThreadRng) -> Option<()> {
        zip.cd.choose_mut(rng)?.version_made_by = rng.gen();
        Some(())
    }
}

struct ModifyCdhCommentLength;

impl ZipMutate for ModifyCdhCommentLength {
    fn mutate(&self, zip: &mut ZipArchive, rng: &mut ThreadRng) -> Option<()> {
        mutate_len(&mut zip.cd.choose_mut(rng)?.file_comment_length, rng);
        Some(())
    }
}

struct ModifyCdhComment;

impl ZipMutate for ModifyCdhComment {
    fn mutate(&self, zip: &mut ZipArchive, rng: &mut ThreadRng) -> Option<()> {
        let cdh = &mut zip.cd.choose_mut(rng)?;
        if cdh.file_comment.is_empty() {
            mutate_len(&mut cdh.file_comment_length, rng);
            cdh.file_comment
                .resize_with(cdh.file_comment_length as usize, || rng.gen());
        } else {
            *cdh.file_comment
                .choose_mut(rng)
                .expect("failed to choose from non-empty comment") = rng.gen();
        }
        Some(())
    }
}

struct ModifyCdhCommentAndLength;

impl ZipMutate for ModifyCdhCommentAndLength {
    fn mutate(&self, zip: &mut ZipArchive, rng: &mut ThreadRng) -> Option<()> {
        let cdh = &mut zip.cd.choose_mut(rng)?;
        mutate_len(&mut cdh.file_comment_length, rng);
        cdh.file_comment
            .resize_with(cdh.file_comment_length as usize, || rng.gen());
        Some(())
    }
}

struct ModifyCdhDiskNumberStart;

impl ZipMutate for ModifyCdhDiskNumberStart {
    fn mutate(&self, zip: &mut ZipArchive, rng: &mut ThreadRng) -> Option<()> {
        let cdh = &mut zip.cd.choose_mut(rng)?;
        mutate_len(&mut cdh.disk_number_start, rng);
        Some(())
    }
}

struct FlipCdhInternalFileAttributes;

impl ZipMutate for FlipCdhInternalFileAttributes {
    fn mutate(&self, zip: &mut ZipArchive, rng: &mut ThreadRng) -> Option<()> {
        let cdh = &mut zip.cd.choose_mut(rng)?;
        let bit = rng.gen_range(0..16);
        cdh.internal_file_attributes ^= InternalFileAttributes::from_bits_retain(1 << bit);
        Some(())
    }
}

struct FlipCdhExternalFileAttributes;

impl ZipMutate for FlipCdhExternalFileAttributes {
    fn mutate(&self, zip: &mut ZipArchive, rng: &mut ThreadRng) -> Option<()> {
        let cdh = &mut zip.cd.choose_mut(rng)?;
        let bit = rng.gen_range(0..32);
        cdh.external_file_attributes ^= 1 << bit;
        Some(())
    }
}

struct ModifyCdhRelativeHeaderOffset;

impl ZipMutate for ModifyCdhRelativeHeaderOffset {
    fn mutate(&self, zip: &mut ZipArchive, rng: &mut ThreadRng) -> Option<()> {
        let from = (0..zip.cd.len()).choose(rng)?;
        let delta = rand_len(rng) as u32;
        let add = rng.gen();
        for cdh in &mut zip.cd[from..] {
            let offset = &mut cdh.relative_header_offset;
            *offset = if add {
                offset.saturating_add(delta)
            } else {
                offset.saturating_sub(delta)
            }
        }
        Some(())
    }
}

struct ModifyContentCompression;

impl ZipMutate for ModifyContentCompression {
    fn mutate(&self, zip: &mut ZipArchive, rng: &mut ThreadRng) -> Option<()> {
        let index = (0..zip.files.len()).choose(rng)?;
        let file = &mut zip.files[index];
        let compression = rand_compression(file.lfh.compression_method, rng);
        let original_data = match decompress(file.lfh.compression_method, &file.data) {
            Ok(decompressed) => Cow::from(decompressed),
            Err(_) => Cow::from(&file.data),
        };
        let compressed = match compress(compression, &original_data) {
            Ok(compressed) => compressed,
            Err(_) => original_data.to_vec(),
        };
        file.lfh.compression_method = compression;
        file.lfh.compressed_size = compressed.len() as _;
        file.lfh.uncompressed_size = original_data.len() as _;
        file.lfh.crc32 = crc32fast::hash(&original_data);
        file.data = compressed;
        if let Some(cdh) = zip.cd.get_mut(index) {
            cdh.compression_method = compression;
            cdh.compressed_size = file.lfh.compressed_size;
            cdh.uncompressed_size = file.lfh.uncompressed_size;
            cdh.crc32 = file.lfh.crc32;
        }
        Some(())
    }
}

struct ModifyContentSize;

impl ZipMutate for ModifyContentSize {
    fn mutate(&self, zip: &mut ZipArchive, rng: &mut ThreadRng) -> Option<()> {
        let index = (0..zip.files.len()).choose(rng)?;
        let file = &mut zip.files[index];
        let compression = file.lfh.compression_method;
        let mut decompressed = decompress(compression, &file.data).ok()?;
        let mut uncompressed_size = decompressed.len();
        mutate_len(&mut uncompressed_size, rng);
        decompressed.resize_with(uncompressed_size, || rng.gen());
        file.data = compress(compression, &decompressed).unwrap();
        file.lfh.uncompressed_size = uncompressed_size as _;
        file.lfh.compressed_size = file.data.len() as _;
        file.lfh.crc32 = crc32fast::hash(&decompressed);
        if let Some(cdh) = zip.cd.get_mut(index) {
            cdh.uncompressed_size = uncompressed_size as _;
            cdh.compressed_size = file.lfh.compressed_size;
            cdh.crc32 = file.lfh.crc32;
        }
        Some(())
    }
}

struct ModifyEocdrCurrentDisk;

impl ZipMutate for ModifyEocdrCurrentDisk {
    fn mutate(&self, zip: &mut ZipArchive, rng: &mut ThreadRng) -> Option<()> {
        mutate_len(&mut zip.eocdr.number_of_this_disk, rng);
        Some(())
    }
}

struct ModifyEocdrStartOfCdDisk;

impl ZipMutate for ModifyEocdrStartOfCdDisk {
    fn mutate(&self, zip: &mut ZipArchive, rng: &mut ThreadRng) -> Option<()> {
        mutate_len(&mut zip.eocdr.start_of_cd_disk_number, rng);
        Some(())
    }
}

struct ModifyEocdrThisDiskCdhCount;

impl ZipMutate for ModifyEocdrThisDiskCdhCount {
    fn mutate(&self, zip: &mut ZipArchive, rng: &mut ThreadRng) -> Option<()> {
        mutate_len(&mut zip.eocdr.this_disk_cdh_count, rng);
        Some(())
    }
}

struct ModifyEocdrTotalCdhCount;

impl ZipMutate for ModifyEocdrTotalCdhCount {
    fn mutate(&self, zip: &mut ZipArchive, rng: &mut ThreadRng) -> Option<()> {
        mutate_len(&mut zip.eocdr.total_cdh_count, rng);
        Some(())
    }
}

fn mutate_cd_size<T>(cd: &[CentralDirectoryHeader], cd_size: &mut T, rng: &mut ThreadRng)
where
    T: Copy + Saturating + Zero + Unsigned + UpperBounded + NumCast,
{
    if cd.is_empty() || rng.gen_ratio(1, 5) {
        mutate_len(cd_size, rng);
    } else {
        *cd_size = T::from(cd[..rng.gen_range(0..cd.len())].byte_count().unwrap())
            .unwrap_or(T::max_value());
    }
}

struct ModifyEocdrCdSize;

impl ZipMutate for ModifyEocdrCdSize {
    fn mutate(&self, zip: &mut ZipArchive, rng: &mut ThreadRng) -> Option<()> {
        mutate_cd_size(&zip.cd, &mut zip.eocdr.size_of_cd, rng);
        Some(())
    }
}

struct ModifyEocdrCdOffset;

impl ZipMutate for ModifyEocdrCdOffset {
    fn mutate(&self, zip: &mut ZipArchive, rng: &mut ThreadRng) -> Option<()> {
        mutate_len(&mut zip.eocdr.offset_of_cd_wrt_starting_disk, rng);
        Some(())
    }
}

struct ModifyEocdrComment;

impl ZipMutate for ModifyEocdrComment {
    fn mutate(&self, zip: &mut ZipArchive, rng: &mut ThreadRng) -> Option<()> {
        if zip.eocdr.zip_file_comment.is_empty() {
            mutate_len(&mut zip.eocdr.zip_file_comment_length, rng);
            zip.eocdr
                .zip_file_comment
                .resize_with(zip.eocdr.zip_file_comment_length as usize, || rng.gen());
        } else {
            *zip.eocdr
                .zip_file_comment
                .choose_mut(rng)
                .expect("failed to choose from non-empty comment") = rng.gen();
        }
        Some(())
    }
}

struct ModifyEocdrCommentLength;

impl ZipMutate for ModifyEocdrCommentLength {
    fn mutate(&self, zip: &mut ZipArchive, rng: &mut ThreadRng) -> Option<()> {
        mutate_len(&mut zip.eocdr.zip_file_comment_length, rng);
        Some(())
    }
}

struct ModifyEocdrCommentAndLength;

impl ZipMutate for ModifyEocdrCommentAndLength {
    fn mutate(&self, zip: &mut ZipArchive, rng: &mut ThreadRng) -> Option<()> {
        mutate_len(&mut zip.eocdr.zip_file_comment_length, rng);
        zip.eocdr
            .zip_file_comment
            .resize_with(zip.eocdr.zip_file_comment_length as usize, || rng.gen());
        Some(())
    }
}

struct UseZip64Eocd;

impl ZipMutate for UseZip64Eocd {
    fn mutate(&self, zip: &mut ZipArchive, _: &mut ThreadRng) -> Option<()> {
        zip.set_eocd(true).ok()
    }
}

struct UseZip64EocdNoFf;

impl ZipMutate for UseZip64EocdNoFf {
    fn mutate(&self, zip: &mut ZipArchive, rng: &mut ThreadRng) -> Option<()> {
        zip.set_eocd(true).ok()?;

        let eocdr =
            EndOfCentralDirectoryRecord::try_from(zip.zip64_eocdr.as_ref().unwrap()).ok()?;

        if rng.gen() {
            zip.eocdr = eocdr;
        } else {
            if rng.gen() {
                zip.eocdr.number_of_this_disk = eocdr.number_of_this_disk;
            }
            if rng.gen() {
                zip.eocdr.start_of_cd_disk_number = eocdr.start_of_cd_disk_number;
            }
            if rng.gen() {
                zip.eocdr.this_disk_cdh_count = eocdr.this_disk_cdh_count;
            }
            if rng.gen() {
                zip.eocdr.total_cdh_count = eocdr.total_cdh_count;
            }
            if rng.gen() {
                zip.eocdr.size_of_cd = eocdr.size_of_cd;
            }
            if rng.gen() {
                zip.eocdr.offset_of_cd_wrt_starting_disk = eocdr.offset_of_cd_wrt_starting_disk;
            }
        }

        Some(())
    }
}

struct ModifyZip64Eocdr;

impl ZipMutate for ModifyZip64Eocdr {
    fn mutate(&self, zip: &mut ZipArchive, rng: &mut ThreadRng) -> Option<()> {
        if zip.zip64_eocdr.is_none() {
            zip.set_eocd(true).ok()?;
        }
        let zip64_eocdr = zip.zip64_eocdr.as_mut().unwrap();
        if rng.gen_ratio(1, 5) {
            mutate_len(&mut zip64_eocdr.size, rng);
        }
        if rng.gen_ratio(1, 5) {
            zip64_eocdr.version_made_by = rng.gen();
        }
        if rng.gen_ratio(1, 5) {
            zip64_eocdr.version_needed = rng.gen();
        }
        if rng.gen_ratio(1, 5) {
            mutate_len(&mut zip64_eocdr.number_of_this_disk, rng);
        }
        if rng.gen_ratio(1, 5) {
            mutate_len(&mut zip64_eocdr.start_of_cd_disk_number, rng);
        }
        if rng.gen_ratio(1, 5) {
            mutate_len(&mut zip64_eocdr.this_disk_cdh_count, rng);
        }
        if rng.gen_ratio(1, 5) {
            mutate_len(&mut zip64_eocdr.total_cdh_count, rng);
        }
        if rng.gen_ratio(1, 5) {
            mutate_cd_size(&zip.cd, &mut zip64_eocdr.size_of_cd, rng);
        }
        if rng.gen_ratio(1, 5) {
            mutate_len(&mut zip64_eocdr.offset_of_cd_wrt_starting_disk, rng);
        }
        Some(())
    }
}

struct UseZip64EocdrV2;

impl ZipMutate for UseZip64EocdrV2 {
    fn mutate(&self, zip: &mut ZipArchive, rng: &mut ThreadRng) -> Option<()> {
        if zip.zip64_eocdr.is_none() {
            zip.set_eocd(true).ok()?;
        }
        let zip64_eocdr = zip.zip64_eocdr.as_mut().unwrap();
        zip64_eocdr.use_v2().expect("failed to use ZIP64 EOCDR v2");
        if rng.gen() {
            zip64_eocdr.version_made_by = 20;
        }
        if rng.gen() {
            zip64_eocdr.version_needed = 20;
        }
        let v2 = zip64_eocdr.v2.as_mut().unwrap();
        if rng.gen_ratio(1, 5) {
            v2.compression_method = CompressionMethod::DEFLATED;
        }
        if rng.gen() {
            mutate_cd_size(&zip.cd, &mut v2.compressed_size, rng);
        }
        if rng.gen() {
            mutate_cd_size(&zip.cd, &mut v2.original_size, rng);
        }
        Some(())
    }
}

struct ModifyEocdl;

impl ZipMutate for ModifyEocdl {
    fn mutate(&self, zip: &mut ZipArchive, rng: &mut ThreadRng) -> Option<()> {
        match zip.zip64_eocdl.as_mut() {
            None => {
                zip.zip64_eocdl = Some(Zip64EndOfCentralDirectoryLocator::from_offset(
                    zip.eocdr.offset_of_cd_wrt_starting_disk.into(),
                ));
            }
            Some(eocdl) => {
                let (a, b, c) = rng.gen();
                if a {
                    mutate_len(&mut eocdl.zip64_eocdr_disk_number, rng);
                }
                if b {
                    mutate_len(&mut eocdl.zip64_eocdr_offset, rng);
                }
                if c {
                    mutate_len(&mut eocdl.total_number_of_disks, rng);
                }
                if !a && !b && !c {
                    zip.zip64_eocdl = None;
                }
            }
        }
        Some(())
    }
}

struct ModifyByte;

impl BytesMutate for ModifyByte {
    fn mutate(&self, bytes: &mut Vec<u8>, rng: &mut ThreadRng) -> Option<()> {
        *bytes.choose_mut(rng)? = rng.gen();
        Some(())
    }
}

struct FlipBit;

impl BytesMutate for FlipBit {
    fn mutate(&self, bytes: &mut Vec<u8>, rng: &mut ThreadRng) -> Option<()> {
        *bytes.choose_mut(rng)? ^= 1 << rng.gen_range(0..8);
        Some(())
    }
}

struct InsertBytes;

impl BytesMutate for InsertBytes {
    fn mutate(&self, bytes: &mut Vec<u8>, rng: &mut ThreadRng) -> Option<()> {
        let len = rand_len(rng);
        let index = (0..=bytes.len()).choose(rng)?;
        let insert = (0..len).map(|_| rng.gen());
        bytes.splice(index..index, insert);
        Some(())
    }
}

struct DeleteBytes;

impl BytesMutate for DeleteBytes {
    fn mutate(&self, bytes: &mut Vec<u8>, rng: &mut ThreadRng) -> Option<()> {
        if bytes.is_empty() {
            return None;
        }
        let len = rand_len(rng).min(bytes.len());
        let index = (0..=(bytes.len() - len)).choose(rng)?;
        bytes.drain(index..index + len);
        Some(())
    }
}

struct DuplicateBytes;

impl BytesMutate for DuplicateBytes {
    fn mutate(&self, bytes: &mut Vec<u8>, rng: &mut ThreadRng) -> Option<()> {
        let r = rand_range(rng, 0..=bytes.len())?;
        let src = bytes[r.0..r.1].to_vec();
        bytes.splice(r.1..r.1, src.clone());
        Some(())
    }
}

struct SpliceBytes;

impl BytesMutate for SpliceBytes {
    fn mutate(&self, bytes: &mut Vec<u8>, rng: &mut ThreadRng) -> Option<()> {
        let src = rand_range(rng, 0..=bytes.len())?;
        let src = bytes[src.0..src.1].to_vec();
        let dst = rand_range(rng, 0..=bytes.len())?;
        bytes.splice(dst.0..dst.1, src);
        Some(())
    }
}
