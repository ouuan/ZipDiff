use crate::config::CONFIG;
use crate::corpus::{Corpus, Seed};
use crate::feature::Feature;
use crate::mutation::{Sample, UcbHandle};
use rayon::prelude::*;
use std::fs;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Instant;
use tar::Builder as TarBuilder;
use walkdir::WalkDir;
use zstd::Encoder as ZstdEncoder;

// An input is (input sample, mutation names, UCB handles)
// Returns the number of actually executed (dedupped) inputs.
pub fn execute(corpus: &mut Corpus, samples: Vec<Sample>) -> Vec<(Vec<UcbHandle>, bool)> {
    save_inputs(&samples);
    let count = samples.len();
    println!("Executing {count} inputs");
    let start = Instant::now();
    run_parsers();
    println!(
        "Finished executing {count} inputs in {:.3} seconds",
        start.elapsed().as_secs_f64()
    );
    let start = Instant::now();
    let ucb_results = collect_results(corpus, samples);
    println!(
        "Collected results in {:.3} seconds",
        start.elapsed().as_secs_f64()
    );
    ucb_results
}

fn save_inputs(samples: &[Sample]) {
    fs::remove_dir_all(&CONFIG.input_dir).ok();
    fs::remove_dir_all(&CONFIG.output_dir).ok();
    fs::create_dir_all(&CONFIG.input_dir).expect("failed to create input dir");

    for sample in samples {
        let path = CONFIG.input_dir.join(&sample.name);
        fs::write(path, &sample.bytes).expect("failed to save input file");
    }
}

fn run_parsers() {
    Command::new("docker")
        .arg("compose")
        .arg("up")
        .current_dir(&CONFIG.parsers_dir)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to run parsers")
        .wait()
        .expect("failed to wait for parsers");
}

fn collect_results(corpus: &mut Corpus, samples: Vec<Sample>) -> Vec<(Vec<UcbHandle>, bool)> {
    samples
        .into_iter()
        .filter_map(|s| {
            let filename = {
                let hash = s.hash.to_string();
                let dir = hash.split_at(2).0;
                format!("{dir}/{hash}.zip")
            };
            let sample_path = CONFIG.samples_dir.join(&filename);
            if sample_path.exists() {
                return None;
            }
            let feat = Feature::par_read(&s.name);
            let interesting = corpus.is_feature_interesting(&feat);
            if interesting {
                fs::create_dir_all(sample_path.parent().unwrap())
                    .expect("failed to create data dir");
                fs::rename(CONFIG.input_dir.join(&s.name), sample_path)
                    .expect("failed to move input file");
                let results_dir = CONFIG.results_dir.join(&filename);
                fs::create_dir_all(&results_dir).expect("failed to create results dir");
                // First move small outputs in parallel with rayon.
                // Then compress large outputs with parallelized ZSTD.
                let large_outputs = CONFIG
                    .parsers
                    .par_iter()
                    .filter_map(|parser| {
                        let output_path = CONFIG.output_dir.join(parser).join(&s.name);
                        let result_path = results_dir.join(parser);
                        if output_path.is_dir() && du(&output_path) > 1024 * 1024 {
                            // tar.zst if larger than 1 MiB
                            Some((result_path.with_extension("tar.zst"), output_path))
                        } else if matches!(output_path.try_exists(), Ok(false)) {
                            fs::write(&result_path, b"").expect(&format!(
                                "failed to write error result to {}",
                                result_path.display(),
                            ));
                            None
                        } else {
                            fs::rename(&output_path, &result_path).expect(&format!(
                                "failed to move {} to {}",
                                output_path.display(),
                                result_path.display()
                            ));
                            None
                        }
                    })
                    .collect::<Vec<_>>();
                large_outputs.iter().for_each(archive_dir);
                corpus.insert_seed(Seed::new(
                    s.input,
                    s.hash,
                    s.bytes.len(),
                    feat,
                    s.mutations,
                    !large_outputs.is_empty(),
                ));
            }
            Some((s.ucb_handles, interesting))
        })
        .collect()
}

fn archive_dir((dest, src): &(impl AsRef<Path>, impl AsRef<Path>)) {
    let file = fs::File::create(dest).expect("failed to create result tar.zst");
    let mut writer = BufWriter::new(file);
    let mut zstd = ZstdEncoder::new(&mut writer, 1).expect("failed to create ZSTD writer");
    zstd.multithread(rayon::current_num_threads() as u32)
        .expect("failed to set multithread ZSTD");
    {
        let mut tar = TarBuilder::new(&mut zstd);
        tar.append_dir_all("", src)
            .expect("failed to archive output");
        tar.finish().expect("failed to finish TAR");
    }
    zstd.finish().expect("failed to finish ZSTD");
    writer.flush().expect("failed to flush output archive");
    // remove here to avoid occupying I/O cache
    fs::remove_dir_all(src).expect("failed to remove output directory");
}

fn du(path: impl AsRef<Path>) -> u64 {
    WalkDir::new(path)
        .into_iter()
        .filter_map(|entry| Some(entry.ok()?.metadata().ok()?.len()))
        .sum()
}
