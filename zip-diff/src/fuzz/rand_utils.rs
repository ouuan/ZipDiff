use crate::CONFIG;
use num_traits::{NumCast, Saturating, Unsigned, Zero};
use rand::distributions::uniform::{SampleRange, SampleUniform};
use rand::distributions::{Standard, WeightedIndex};
use rand::prelude::*;
use zip_diff::zip::ZipArchive;

pub struct Ucb {
    scores: Vec<f64>,
    trials: Vec<f64>,
    weighted_index: Option<WeightedIndex<f64>>,
}

impl Ucb {
    pub fn new(len: usize) -> Self {
        Self {
            scores: vec![0.0; len],
            trials: vec![0.0; len],
            weighted_index: None,
        }
    }

    pub fn construct(&mut self) {
        for i in 0..self.scores.len() {
            // recent results are more important than old results
            self.scores[i] *= 0.995;
            self.trials[i] *= 0.995;
        }
        let double_ln_total_trial: f64 = 2.0 * self.trials.iter().sum::<f64>().max(1.0).ln();
        let weights = self
            .scores
            .iter()
            .zip(self.trials.iter().map(|t| t.max(1.0)))
            .map(|(score, trial)| {
                let ucb = score / trial + (double_ln_total_trial / trial).sqrt();
                (ucb * 5.0).exp() // softmax temperature
            });
        self.weighted_index = if CONFIG.argmax_ucb {
            let mut max_weight = f64::NEG_INFINITY;
            for w in weights.clone() {
                if w > max_weight {
                    max_weight = w;
                }
            }
            Some(
                WeightedIndex::new(weights.map(|w| {
                    if w == max_weight {
                        1.0
                    } else {
                        1e-6 // not zero to avoid loop when always fail to mutate
                    }
                }))
                .unwrap(),
            )
        } else {
            Some(WeightedIndex::new(weights).unwrap())
        };
    }

    pub fn sample<R: Rng>(&self, rng: &mut R) -> usize {
        self.weighted_index
            .as_ref()
            .expect("need to construt before sampling")
            .sample(rng)
    }

    pub fn record(&mut self, id: usize, trial: f64, score: f64) {
        self.trials[id] += trial;
        self.scores[id] += score;
        self.weighted_index = None;
    }

    pub fn scores(&self) -> &[f64] {
        &self.scores
    }

    pub fn trials(&self) -> &[f64] {
        &self.trials
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum HeaderLocation {
    Lfh,
    Cdh,
    Both,
}

impl HeaderLocation {
    pub fn lfh(self) -> bool {
        matches!(self, Self::Lfh | Self::Both)
    }

    pub fn cdh(self) -> bool {
        matches!(self, Self::Cdh | Self::Both)
    }
}

impl Distribution<HeaderLocation> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> HeaderLocation {
        let i = (0..5).choose(rng).unwrap();
        match i {
            0 => HeaderLocation::Lfh,
            1 => HeaderLocation::Cdh,
            _ => HeaderLocation::Both,
        }
    }
}

pub fn rand_header<R: RngCore>(zip: &ZipArchive, rng: &mut R) -> Option<(usize, HeaderLocation)> {
    let loc = rng.gen();

    let len = match loc {
        HeaderLocation::Lfh => zip.files.len(),
        HeaderLocation::Cdh => zip.cd.len(),
        HeaderLocation::Both => zip.files.len().min(zip.cd.len()),
    };

    let index = (0..len).choose(rng)?;

    Some((index, loc))
}

/// returns a random number in 1..=32, returns x with possibility 2^-x
pub fn rand_len<R: RngCore>(rng: &mut R) -> usize {
    rng.next_u64().trailing_zeros() as usize + 1
}

pub fn mutate_len<R, N>(size: &mut N, rng: &mut R)
where
    R: RngCore,
    N: Copy + Saturating + Zero + Unsigned + NumCast,
{
    let delta = N::from(rand_len(rng)).unwrap();
    if size.is_zero() || rng.gen() {
        *size = size.saturating_add(delta);
    } else {
        *size = size.saturating_sub(delta);
    }
}

pub fn rand_range<G, T, R>(rng: &mut G, range: R) -> Option<(T, T)>
where
    G: Rng,
    T: SampleUniform + Ord,
    R: SampleRange<T> + Clone,
{
    if range.is_empty() {
        return None;
    }
    let x = rng.gen_range(range.clone());
    let y = rng.gen_range(range);
    if x < y {
        Some((x, y))
    } else {
        Some((y, x))
    }
}
