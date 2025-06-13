use crate::config::CONFIG;
use fixedbitset::FixedBitSet;
use std::ops::BitOrAssign;
use std::path::Path;
use std::sync::LazyLock;
use zip_diff::hash::{read_parsing_result, ParsingResult};

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Feature {
    pub ok: FixedBitSet,
    pub inconsistency: FixedBitSet,
}

pub static PAIR_LIST: LazyLock<Vec<(String, String)>> = LazyLock::new(Feature::pair_list);

impl Feature {
    pub fn new() -> Self {
        let n = CONFIG.parsers.len();
        let ok = FixedBitSet::with_capacity(n);
        let inconsistency = FixedBitSet::with_capacity(n * (n - 1) / 2);
        Self { ok, inconsistency }
    }

    pub fn par_read(name: impl AsRef<Path>) -> Self {
        let mut feature = Self::new();
        feature.apply_testcase(name, true);
        feature
    }

    pub fn apply_testcase(&mut self, name: impl AsRef<Path>, par: bool) {
        let results = CONFIG
            .parsers
            .iter()
            .map(|parser| read_parsing_result(CONFIG.output_dir.join(parser).join(&name), par))
            .collect::<Vec<_>>();

        let mut p = 0;
        for (i, x) in results.iter().enumerate() {
            if matches!(x, ParsingResult::Ok(_)) {
                self.ok.insert(i);
            }
            for y in &results[..i] {
                if x.inconsistent_with(y) {
                    self.inconsistency.insert(p);
                }
                p += 1;
            }
        }
    }

    pub fn is_covered_by(&self, by: &Self) -> bool {
        self.inconsistency.is_subset(&by.inconsistency) && self.ok.is_subset(&by.ok)
    }

    pub fn consistent_pairs(&self) -> Vec<&'static (String, String)> {
        self.inconsistency.zeroes().map(|i| &PAIR_LIST[i]).collect()
    }

    pub fn summary(&self) -> String {
        let ok_count = self.ok.count_ones(..);
        let incons_count = self.inconsistency.count_ones(..);
        format!("ok: {ok_count:2}, incons: {incons_count:4}")
    }

    fn pair_list() -> Vec<(String, String)> {
        let mut result = Vec::new();
        for (i, x) in CONFIG.parsers.iter().enumerate() {
            for y in CONFIG.parsers.iter().take(i) {
                result.push((x.clone(), y.clone()));
            }
        }
        result
    }
}

impl BitOrAssign<&Feature> for Feature {
    fn bitor_assign(&mut self, rhs: &Feature) {
        self.ok |= &rhs.ok;
        self.inconsistency |= &rhs.inconsistency;
    }
}
