use std::collections::BTreeMap;
use std::collections::HashMap;
use std::io;

use prometheus::proto::MetricFamily;

use prometheus_parse::Labels;
use prometheus_parse::Sample;
use prometheus_parse::Scrape;
use prometheus_parse::Value;

pub trait ProtoMerge {
    type Error: core::error::Error;

    fn merge(&self, state: Vec<MetricFamily>, next: Vec<MetricFamily>) -> Vec<MetricFamily>;
}

pub trait ParseMerge {
    fn merge(&self, state: Scrape, next: Scrape) -> Scrape;

    fn sources2merged<S, N>(
        &self,
        mut state: S,
        mut next: N,
    ) -> impl FnMut() -> Result<Scrape, io::Error>
    where
        S: FnMut() -> Result<Scrape, io::Error>,
        N: FnMut() -> Result<Scrape, io::Error>,
    {
        move || {
            let s: Scrape = state()?;
            let n: Scrape = next()?;
            let merged: Scrape = self.merge(s, n);
            Ok(merged)
        }
    }
}

impl<F> ParseMerge for F
where
    F: Fn(Scrape, Scrape) -> Scrape,
{
    fn merge(&self, state: Scrape, next: Scrape) -> Scrape {
        self(state, next)
    }
}

pub fn gauge_merge(_state: f64, next: f64) -> Value {
    Value::Gauge(next)
}

pub fn counter_merge(state: f64, next: f64) -> Value {
    Value::Counter(state + next)
}

pub fn value_merge(state: Value, next: Value) -> Value {
    match (state, next) {
        (Value::Counter(s), Value::Counter(n)) => counter_merge(s, n),
        (Value::Gauge(s), Value::Gauge(n)) => gauge_merge(s, n),
        (_, others) => others,
    }
}

pub fn sample_merge(state: Sample, next: Sample) -> Sample {
    let sval: Value = state.value;
    let nval: Value = next.value;
    let merged: Value = value_merge(sval, nval);
    Sample {
        metric: next.metric,
        value: merged,
        labels: next.labels,
        timestamp: next.timestamp,
    }
}

pub fn labels2tree(l: Labels) -> BTreeMap<String, String> {
    let hr: &HashMap<String, String> = &l;
    let h: HashMap<_, _> = hr.clone();
    BTreeMap::from_iter(h)
}

#[derive(PartialEq, Eq, Hash)]
pub struct LabelTree(pub BTreeMap<String, String>);

#[derive(PartialEq, Eq, Hash)]
pub struct HashKey(pub String, pub LabelTree);

pub fn sample2key(s: &Sample) -> HashKey {
    let metric: String = s.metric.clone();
    let labels: Labels = s.labels.clone();
    let ltree: LabelTree = LabelTree(labels2tree(labels));
    HashKey(metric, ltree)
}

pub fn samples2hash(s: Vec<Sample>) -> HashMap<HashKey, Sample> {
    HashMap::from_iter(s.into_iter().map(|s| (sample2key(&s), s)))
}

pub fn scrape_merge(state: Scrape, next: Scrape) -> Scrape {
    let sdocs: HashMap<String, String> = state.docs;
    let ndocs: HashMap<String, String> = next.docs;
    let mut merged = ndocs;
    merged.extend(sdocs);

    let smet: Vec<Sample> = state.samples;
    let nmet: Vec<Sample> = next.samples;

    let mut smap: HashMap<HashKey, Sample> = samples2hash(smet);
    let nmap: HashMap<HashKey, Sample> = samples2hash(nmet);

    let mut merged_samples: Vec<Sample> = vec![];

    for (key, nsamp) in nmap.into_iter() {
        let osamp: Option<Sample> = smap.remove(&key);
        let merged: Sample = match osamp {
            None => nsamp,
            Some(st) => sample_merge(st, nsamp),
        };
        merged_samples.push(merged);
    }

    for (_, st) in smap.into_iter() {
        merged_samples.push(st);
    }

    Scrape {
        docs: merged,
        samples: merged_samples,
    }
}

pub fn state_next2merged<S, N>(state: S, next: N) -> impl FnMut() -> Result<Scrape, io::Error>
where
    S: FnMut() -> Result<Scrape, io::Error>,
    N: FnMut() -> Result<Scrape, io::Error>,
{
    scrape_merge.sources2merged(state, next)
}
