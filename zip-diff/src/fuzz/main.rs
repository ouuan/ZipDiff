#![allow(clippy::expect_fun_call)]

mod config;
mod corpus;
mod execute;
mod feature;
mod generate;
mod mutation;
mod rand_utils;
mod stats;

use binwrite::BinWrite;
use config::CONFIG;
use corpus::Corpus;
use execute::execute;
use mutation::Mutator;
use rand::thread_rng;
use stats::Stats;
use std::fs::canonicalize;
use std::process::Command;
use std::time::Instant;
use zip_diff::zip::ZipArchive;

#[derive(Clone)]
pub enum Input {
    Zip(Box<ZipArchive>),
    Bytes(Vec<u8>),
}

fn main() {
    let input_dir = canonicalize(&CONFIG.input_dir).expect("failed to canonicalize input dir");
    let output_dir = canonicalize(&CONFIG.output_dir).expect("failed to canonicalize output dir");
    let parser_prepare_status = Command::new(CONFIG.parsers_dir.join("prepare.sh"))
        .env("INPUT_DIR", input_dir)
        .env("OUTPUT_DIR", output_dir)
        .status()
        .expect("failed to execute prepare.sh");
    assert!(parser_prepare_status.success(), "prepare.sh failed");

    let mut mutator = Mutator::new();
    let mut stats = Stats::new();
    let mut corpus = Corpus::new();
    let rng = &mut thread_rng();

    let initial_samples = generate::init_corpus()
        .into_iter()
        .filter_map(|zip| {
            let input = if CONFIG.byte_mutation_only {
                let mut buf = Vec::new();
                zip.write(&mut buf)
                    .expect("failed to convert initial ZIP to bytes");
                Input::Bytes(buf)
            } else {
                Input::Zip(Box::new(zip))
            };
            let sample = mutator.generate_sample(&input, &[], 0, rng);
            corpus.insert_hash(sample.hash).then_some(sample)
        })
        .collect();

    execute(&mut corpus, initial_samples);

    loop {
        println!(
            "inputs: {}, corpus size: {} ({} zips), sum: {}",
            stats.input_count(),
            corpus.len(),
            corpus.zip_count(),
            corpus.feature_sum_summary()
        );
        corpus.construct_weights();
        mutator.construct_ucb();
        let (seed_indices, samples): (Vec<_>, Vec<_>) = std::iter::repeat(())
            .filter_map(|_| {
                let (seed_index, seed) = corpus.select_seed(rng);
                let mutate_times = rand_utils::rand_len(rng);
                let sample =
                    mutator.generate_sample(&seed.input, &seed.mutations, mutate_times, rng);
                corpus
                    .insert_hash(sample.hash)
                    .then_some((seed_index, sample))
            })
            .take(CONFIG.batch_size)
            .unzip();
        for index in seed_indices {
            corpus.record_selection(index);
        }
        let ucb_results = execute(&mut corpus, samples);
        mutator.record_ucb(&ucb_results);
        stats.record_iteration(ucb_results.len(), &corpus, &mutator);
        stats.save();
        if let Some(stop_at) = CONFIG.stop_at {
            if Instant::now() > stop_at {
                break;
            }
        }
    }
}
