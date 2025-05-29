use crate::config::CONFIG;
use crate::corpus::Corpus;
use crate::mutation::{MutationStats, Mutator};
use serde::Serialize;
use std::collections::BTreeMap;
use std::fs::File;
use std::time::Instant;

#[derive(Serialize)]
struct Iteration {
    input_count: usize,
    corpus_size: usize,
    incons_count: usize,
    seconds_used: f64,
}

#[derive(Serialize)]
struct SeedStat {
    hash: String,
    mutations: Vec<&'static str>,
    ok_count: usize,
    incons_count: usize,
    selection_count: usize,
}

#[derive(Serialize)]
pub struct Stats {
    #[serde(skip)]
    start_at: Instant,
    /// total number of generated inputs
    input_count: usize,
    /// hash of best seeds
    best_seeds: Vec<SeedStat>,
    /// map from parser pair to best seed hash
    best_seed_map: BTreeMap<&'static String, BTreeMap<&'static String, String>>,
    /// fuzzing iteration history
    iterations: Vec<Iteration>,
    /// parser pairs that are consistent in the test cases
    consistent_pairs: Vec<&'static (String, String)>,
    /// Mutation trials
    mutations: Option<MutationStats>,
    // ablation configs
    argmax_ucb: bool,
    byte_mutation_only: bool,
}

impl Stats {
    pub fn new() -> Self {
        Self {
            start_at: Instant::now(),
            input_count: 0,
            best_seeds: Vec::new(),
            best_seed_map: BTreeMap::new(),
            iterations: Vec::new(),
            consistent_pairs: Vec::new(),
            mutations: None,
            argmax_ucb: CONFIG.argmax_ucb,
            byte_mutation_only: CONFIG.byte_mutation_only,
        }
    }

    pub fn record_iteration(&mut self, new_input_count: usize, corpus: &Corpus, mutator: &Mutator) {
        self.input_count += new_input_count;
        let mut best_seeds = Vec::new();
        self.best_seed_map = BTreeMap::new();
        for (a, b, seed) in corpus.best_seeds() {
            self.best_seed_map
                .entry(a)
                .or_default()
                .insert(b, seed.hash.to_string());
            best_seeds.push(seed);
        }
        best_seeds.sort_unstable_by_key(|seed| seed.hash.as_bytes());
        best_seeds.dedup_by_key(|seed| &seed.hash);
        self.best_seeds = best_seeds
            .into_iter()
            .map(|seed| SeedStat {
                hash: seed.hash.to_string(),
                mutations: seed.mutations.clone(),
                ok_count: seed.feat.ok.count_ones(..),
                incons_count: seed.feat.inconsistency.count_ones(..),
                selection_count: seed.selection_count,
            })
            .collect();
        self.iterations.push(Iteration {
            input_count: self.input_count,
            corpus_size: corpus.len(),
            incons_count: corpus.incons_count(),
            seconds_used: self.start_at.elapsed().as_secs_f64(),
        });
        self.consistent_pairs = corpus.consistent_pairs();
        self.mutations = Some(mutator.stats());
    }

    pub fn input_count(&self) -> usize {
        self.input_count
    }

    pub fn save(&self) {
        let file = File::create(&CONFIG.stats_file).expect("failed to create stats file");
        serde_json::to_writer_pretty(file, self).expect("failed to write stats");
    }
}
