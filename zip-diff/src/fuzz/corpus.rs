use crate::feature::{Feature, PAIR_LIST};
use crate::Input;
use blake3::Hash;
use rand::distributions::WeightedIndex;
use rand::prelude::*;
use rayon::prelude::*;
use std::cmp::Reverse;
use std::collections::HashSet;

pub struct Seed {
    pub input: Input,
    pub hash: Hash,
    pub size: usize,
    pub feat: Feature,
    pub mutations: Vec<&'static str>,
    pub output_large: bool,
    pub selection_count: usize,
    pub fixed_energy: f64,
}

impl Seed {
    pub fn new(
        input: Input,
        hash: Hash,
        size: usize,
        feat: Feature,
        mutations: Vec<&'static str>,
        output_large: bool,
    ) -> Self {
        let mutation_count_energy = (-(mutations.len() as f64) / 4.0).exp();
        let size_energy = 100.0 / size as f64;
        let ok_energy = feat.ok.count_ones(..) as f64 / feat.ok.len() as f64;
        Self {
            input,
            hash,
            size,
            feat,
            mutations,
            output_large,
            selection_count: 0,
            fixed_energy: mutation_count_energy + size_energy + ok_energy,
        }
    }
}

pub struct Corpus {
    seeds: Vec<Seed>,
    feature_sum: Feature,
    hash_set: HashSet<Hash>,
    weighted_index: Option<WeightedIndex<f64>>,
}

impl Corpus {
    pub fn new() -> Self {
        Self {
            seeds: Vec::new(),
            feature_sum: Feature::new(),
            hash_set: HashSet::new(),
            weighted_index: None,
        }
    }

    pub fn len(&self) -> usize {
        self.seeds.len()
    }

    pub fn zip_count(&self) -> usize {
        self.seeds
            .iter()
            .filter(|seed| matches!(seed.input, Input::Zip(_)))
            .count()
    }

    pub fn incons_count(&self) -> usize {
        self.feature_sum.inconsistency.count_ones(..)
    }

    pub fn feature_sum_summary(&self) -> String {
        self.feature_sum.summary()
    }

    pub fn consistent_pairs(&self) -> Vec<&'static (String, String)> {
        self.feature_sum.consistent_pairs()
    }

    pub fn best_seeds(&self) -> impl Iterator<Item = (&'static String, &'static String, &Seed)> {
        self.feature_sum.inconsistency.ones().map(|i| {
            let best = self
                .seeds
                .iter()
                .filter(|seed| seed.feat.inconsistency.contains(i))
                .max_by_key(|seed| {
                    (
                        Reverse(seed.mutations.len()),
                        seed.feat.inconsistency.count_ones(..),
                        seed.feat.ok.count_ones(..),
                        Reverse(seed.size),
                    )
                })
                .unwrap();
            let (a, b) = &PAIR_LIST[i];
            (a, b, best)
        })
    }

    pub fn insert_hash(&mut self, hash: Hash) -> bool {
        self.hash_set.insert(hash)
    }

    pub fn is_feature_interesting(&self, feat: &Feature) -> bool {
        self.seeds
            .par_iter()
            .all(|old| !feat.is_covered_by(&old.feat))
    }

    pub fn insert_seed(&mut self, seed: Seed) {
        self.feature_sum |= &seed.feat;
        self.weighted_index = None;
        self.seeds.retain(|old| !old.feat.is_covered_by(&seed.feat));
        self.seeds.push(seed);
    }

    pub fn construct_weights(&mut self) {
        let incons_popularity = self
            .seeds
            .par_iter()
            .fold_with(
                vec![0usize; self.feature_sum.inconsistency.len()],
                |mut sum, seed| {
                    seed.feat.inconsistency.ones().for_each(|i| sum[i] += 1);
                    sum
                },
            )
            .reduce_with(|mut a, b| {
                a.iter_mut().zip(b).for_each(|(x, y)| *x += y);
                a
            })
            .unwrap();
        let incons_energy_coef =
            self.seeds.len() as f64 / self.feature_sum.inconsistency.count_ones(..) as f64;
        let weights = self
            .seeds
            .par_iter()
            .map(|seed| {
                let selection_energy = (-(seed.selection_count as f64) / 4.0).exp();
                let incons_energy = seed
                    .feat
                    .inconsistency
                    .ones()
                    .map(|i| incons_energy_coef / incons_popularity[i] as f64)
                    .sum::<f64>();
                let energy = seed.fixed_energy + incons_energy + selection_energy;
                if seed.output_large {
                    energy / 10.0
                } else {
                    energy
                }
            })
            .collect::<Vec<_>>();
        self.weighted_index = Some(WeightedIndex::new(weights).expect("invalid weights"));
    }

    pub fn select_seed(&self, rng: &mut ThreadRng) -> (usize, &Seed) {
        let index = self
            .weighted_index
            .as_ref()
            .expect("weights not constructed")
            .sample(rng);
        (index, &self.seeds[index])
    }

    pub fn record_selection(&mut self, index: usize) {
        self.seeds[index].selection_count += 1;
    }
}
