use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsString;
use std::fs::{copy, create_dir_all, read_dir, remove_dir_all, File};
use std::io::{BufReader, BufWriter, ErrorKind, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use zip_diff::hash::read_parsing_result;

const SAMPLES_DIR: &str = "../constructions";
const INPUT_DIR: &str = "../constructions/input";
const OUTPUT_DIR: &str = "../constructions/output";

const TYPES: &[&str] = &[
    "a1", "a2", "a3", "a4", "a5", // Redundant Metadata
    "b1", "b2", "b3", "b4", // File Path Processing
    "c1", "c2", "c3", "c4", "c5", // ZIP Structure Positioning
];

#[derive(Serialize)]
struct InconsistencyItem<'a> {
    parsers: (&'a str, &'a str),
    inconsistency_types: Vec<&'static str>,
}

#[derive(Deserialize)]
pub struct ParserInfo {
    pub name: String,
    pub version: String,
    pub r#type: String,
    pub language: String,
}

fn main() -> Result<()> {
    let parsers_file =
        File::open("../parsers/parsers.json").context("failed to read parsers.json")?;
    let parsers_reader = BufReader::new(parsers_file);
    let parser_map: BTreeMap<String, ParserInfo> = serde_json::from_reader(parsers_reader)?;
    let mut parsers = parser_map.into_iter().collect::<Vec<_>>();
    parsers.sort_by_cached_key(|(_, parser)| {
        (
            parser.r#type.clone(),
            parser.language.clone(),
            parser.name.to_lowercase(),
            parser.version.clone(),
        )
    });
    let parsers = parsers.into_iter().map(|(key, _)| key).collect::<Vec<_>>();

    if let Err(err) = remove_dir_all(INPUT_DIR) {
        if err.kind() != ErrorKind::NotFound {
            bail!("failed to remove input directory: {err}");
        }
    }
    if let Err(err) = remove_dir_all(OUTPUT_DIR) {
        if err.kind() != ErrorKind::NotFound {
            bail!("failed to remove output directory: {err}");
        }
    }
    create_dir_all(INPUT_DIR).context("failed to remove input directory")?;

    let mut testcases = Vec::<(&str, OsString)>::new();

    for t in TYPES {
        let dir = Path::new(SAMPLES_DIR).join(t);
        if !dir.try_exists()? {
            continue;
        }
        let entries = read_dir(dir)?;
        for entry in entries {
            let entry = entry?;
            if entry.file_name().into_string().unwrap().starts_with(t)
                && entry.file_type()?.is_file()
            {
                testcases.push((t, entry.file_name()));
                copy(entry.path(), Path::new(INPUT_DIR).join(entry.file_name()))
                    .context("failed to copy sample to input directory")?;
            }
        }
    }

    let parser_prepare_status = Command::new("../parsers/prepare.sh")
        .env("INPUT_DIR", INPUT_DIR)
        .env("OUTPUT_DIR", OUTPUT_DIR)
        .status()
        .expect("failed to execute parsers/prepare.sh");
    if !parser_prepare_status.success() {
        bail!("parsers/prepare.sh failed");
    }

    Command::new("docker")
        .arg("compose")
        .arg("up")
        .current_dir("../parsers")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .context("failed to start docker compose")?
        .wait()
        .context("failed to run docker compose")?;

    let outputs = parsers
        .iter()
        .map(|parser| {
            testcases
                .iter()
                .map(|(_, t)| read_parsing_result(Path::new(OUTPUT_DIR).join(parser).join(t), true))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    let mut results = Vec::new();

    for (parser1, outputs1) in parsers.iter().zip(&outputs) {
        for (parser2, outputs2) in parsers.iter().zip(&outputs) {
            let inconsistency_types = outputs1
                .iter()
                .zip(outputs2)
                .zip(&testcases)
                .filter_map(|((o1, o2), (t, _))| o1.inconsistent_with(o2).then_some(*t))
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect();
            results.push(InconsistencyItem {
                parsers: (parser1, parser2),
                inconsistency_types,
            })
        }
    }

    let results_file = File::create(Path::new(SAMPLES_DIR).join("inconsistency-types.json"))
        .context("failed to create result file")?;
    let mut results_writer = BufWriter::new(results_file);
    serde_json::to_writer_pretty(&mut results_writer, &results)
        .context("failed to write results")?;
    results_writer.flush()?;

    Ok(())
}
