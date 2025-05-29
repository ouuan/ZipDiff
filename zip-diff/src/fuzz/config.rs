use clap::Parser;
use fs4::available_space;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs::{create_dir_all, File};
use std::path::PathBuf;
use std::sync::LazyLock;
use std::time::{Duration, Instant};
use sysinfo::System;

pub struct Config {
    pub batch_size: usize,
    pub parsers: Vec<String>,
    pub parsers_dir: PathBuf,
    pub input_dir: PathBuf,
    pub output_dir: PathBuf,
    pub samples_dir: PathBuf,
    pub results_dir: PathBuf,
    pub stats_file: PathBuf,
    pub argmax_ucb: bool,
    pub byte_mutation_only: bool,
    pub stop_at: Option<Instant>,
}

pub static CONFIG: LazyLock<Config> = LazyLock::new(|| {
    let opts = Cli::parse();

    create_dir_all(&opts.input_dir).expect("failed to create input dir");
    create_dir_all(&opts.output_dir).expect("failed to create output dir");
    let batch_size = opts.batch_size.unwrap_or_else(|| default_batch_size(&opts));

    let parsers_dir = PathBuf::from(opts.parsers_dir);
    let input_dir = PathBuf::from(opts.input_dir);
    let output_dir = PathBuf::from(opts.output_dir);
    let samples_dir = PathBuf::from(opts.samples_dir);
    let results_dir = PathBuf::from(opts.results_dir);

    let stats_file = PathBuf::from(opts.stats_file);
    create_dir_all(stats_file.parent().expect("stats file path has no parent"))
        .expect("failed to create parent dir for stats file");

    let parsers_file =
        File::open(parsers_dir.join("parsers.json")).expect("failed to open parsers.json");
    let parser_map: BTreeMap<String, ParserInfo> =
        serde_json::from_reader(parsers_file).expect("failed to read parsers.json");

    let stop_at = opts
        .stop_after_seconds
        .map(|secs| Instant::now() + Duration::from_secs(secs));

    Config {
        batch_size,
        parsers: parser_map.into_keys().collect(),
        parsers_dir,
        input_dir,
        output_dir,
        samples_dir,
        results_dir,
        stats_file,
        argmax_ucb: opts.argmax_ucb,
        byte_mutation_only: opts.byte_mutation_only,
        stop_at,
    }
});

fn default_batch_size(opts: &Cli) -> usize {
    let mut sys = System::new();
    sys.refresh_memory();
    let ram = sys.total_memory();
    let ram_batch_size = ram.div_ceil(1024 * 1024 * 1024).saturating_sub(20) as usize;
    if ram_batch_size < 100 {
        eprintln!("Warning: Available RAM is below the recommended minimum");
    }
    let disk =
        available_space(&opts.output_dir).expect("failed to get available space for output dir");
    let disk_batch_size = disk.div_ceil(2 * 1024 * 1024 * 1024) as usize;
    if disk_batch_size < 100 {
        eprintln!("Warning: Available disk space is below the recommended minimum");
    }
    ram_batch_size.min(disk_batch_size)
}

#[derive(Parser)]
struct Cli {
    /// number of samples per execution batch [default: depends on available resources]
    #[arg(short, long)]
    batch_size: Option<usize>,
    /// Stop running after how many seconds [default: infinite]
    #[arg(short, long)]
    stop_after_seconds: Option<u64>,
    /// directory to find the parsers
    #[arg(long, default_value = "../parsers")]
    parsers_dir: String,
    /// directory to temporarily save input samples for parsers in Docker
    #[arg(long, default_value = "../evaluation/input")]
    input_dir: String,
    /// directory to temporarily save temporary outputs for parsers in Docker
    #[arg(long, default_value = "../evaluation/output")]
    output_dir: String,
    /// directory to store interesting samples
    #[arg(long, default_value = "../evaluation/samples")]
    samples_dir: String,
    /// directory to store outputs of interesting samples
    #[arg(long, default_value = "../evaluation/results")]
    results_dir: String,
    /// file to save the fuzz stats
    #[arg(long, default_value = "../evaluation/stats.json")]
    stats_file: String,
    /// Use argmax UCB instead of softmax UCB
    #[arg(long, default_value_t = false)]
    argmax_ucb: bool,
    /// Use byte-level mutations only without ZIP-level mutations
    #[arg(long, default_value_t = false)]
    byte_mutation_only: bool,
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct ParserInfo {
    name: String,
    version: String,
}
