use blake3::{Hash, Hasher};
use std::path::Path;

#[derive(Clone, Copy)]
pub enum ParsingResult {
    Ok(Hash),
    Err,
}

impl ParsingResult {
    pub fn inconsistent_with(&self, rhs: &Self) -> bool {
        match (self, rhs) {
            (ParsingResult::Ok(lhs), ParsingResult::Ok(rhs)) => lhs != rhs,
            _ => false,
        }
    }
}

pub fn read_parsing_result(path: impl AsRef<Path>, par: bool) -> ParsingResult {
    let path = path.as_ref();
    if path.is_dir() {
        ParsingResult::Ok(dirhash(path, par).unwrap_or(Hash::from_bytes(Default::default())))
    } else {
        ParsingResult::Err
    }
}

// Returns `None` for empty directory
fn dirhash(path: impl AsRef<Path>, par: bool) -> Option<Hash> {
    let path = path.as_ref();
    let path_display = path.display();
    let mut hasher = Hasher::new();

    if path.is_symlink() {
        hasher.update(b"L");
        hasher.update(
            &path
                .read_link()
                .unwrap_or_else(|_| panic!("failed to read link {path_display}"))
                .into_os_string()
                .into_encoded_bytes(),
        );
    } else if path.is_file() {
        hasher.update(b"F");
        if par {
            hasher.update_mmap_rayon(path)
        } else {
            hasher.update_mmap(path)
        }
        .unwrap_or_else(|_| panic!("failed to read file {path_display}"));
    } else if path.is_dir() {
        hasher.update(b"D");
        let mut children = path
            .read_dir()
            .unwrap_or_else(|_| panic!("failed to read dir {path_display}"))
            .filter_map(|entry| {
                let entry =
                    entry.unwrap_or_else(|_| panic!("failed to read dir entry in {path_display}"));
                let entry_path = entry.path();
                let mut hasher = Hasher::new();
                let name = entry.file_name().into_encoded_bytes();
                if name.iter().all(|x| {
                    x.is_ascii_alphanumeric() || matches!(x, b'.' | b'_' | b'-' | b'[' | b']')
                }) {
                    hasher.update(b"N");
                    hasher.update(&name);
                } else {
                    // treat all special file names as the same
                    hasher.update(b"S");
                }
                hasher.update(
                    dirhash(entry_path, par)? /* ignore empty dir */
                        .as_bytes(),
                );
                Some(hasher.finalize().into())
            })
            .collect::<Vec<[u8; 32]>>();
        if children.is_empty() {
            return None;
        }
        children.sort_unstable();
        for child in children {
            hasher.update(&child);
        }
    } else {
        panic!("file does not exist, permission error, or unknown file type: {path_display}");
    }

    Some(hasher.finalize())
}
