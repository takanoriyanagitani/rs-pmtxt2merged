use std::collections::HashMap;
use std::io;

use prometheus::proto::MetricFamily;
use prometheus::proto::MetricType;
use prometheus_parse::{Sample, Scrape, Value};

use crate::ser::{BasicCounter, BasicFamily, BasicGauge, BasicLabel, BasicMetric};

pub trait ParsedToProto {
    type Error: core::error::Error;

    fn convert(&self, parsed: Scrape) -> Result<Vec<MetricFamily>, Self::Error>;
}

impl<F, E> ParsedToProto for F
where
    F: Fn(Scrape) -> Result<Vec<MetricFamily>, E>,
    E: core::error::Error,
{
    type Error = E;
    fn convert(&self, parsed: Scrape) -> Result<Vec<MetricFamily>, Self::Error> {
        self(parsed)
    }
}

pub fn scrape2basic(s: Scrape) -> Result<Vec<BasicFamily>, io::Error> {
    let mut grouped_samples: HashMap<String, Vec<Sample>> = HashMap::new();
    for sample in s.samples {
        grouped_samples
            .entry(sample.metric.clone())
            .or_default()
            .push(sample);
    }

    let mut families: Vec<BasicFamily> = Vec::with_capacity(grouped_samples.len());

    for (name, samples) in grouped_samples {
        if samples.is_empty() {
            continue;
        }

        let oftyp: Option<MetricType> = match &samples[0].value {
            Value::Counter(_) => Some(MetricType::COUNTER),
            Value::Gauge(_) => Some(MetricType::GAUGE),
            _ => None,
        };

        let Some(ftyp) = oftyp else {
            continue;
        };

        let help = s.docs.get(&name).cloned().unwrap_or_default();

        let mut metrics: Vec<BasicMetric> = Vec::with_capacity(samples.len());
        for sample in samples {
            let labels: Vec<BasicLabel> = sample
                .labels
                .iter()
                .map(|(k, v)| BasicLabel {
                    name: k.into(),
                    value: v.into(),
                })
                .collect();

            match sample.value {
                Value::Counter(v) => {
                    metrics.push(BasicMetric::from_counter(labels, BasicCounter(v)));
                }
                Value::Gauge(v) => {
                    metrics.push(BasicMetric::from_gauge(labels, BasicGauge(v)));
                }
                _ => {}
            }
        }

        families.push(BasicFamily {
            name,
            ftyp,
            help,
            met: metrics,
        });
    }

    Ok(families)
}

pub fn basic2proto(b: Vec<BasicFamily>) -> Vec<MetricFamily> {
    BasicFamily::convert(b)
}

pub fn scrape2proto(s: Scrape) -> Result<Vec<MetricFamily>, io::Error> {
    let vb: Vec<BasicFamily> = scrape2basic(s)?;
    Ok(basic2proto(vb))
}

pub struct InOut<I, O> {
    pub input_scrape: I,
    pub output_proto: O,
}

impl<I, O> InOut<I, O>
where
    I: FnMut() -> Result<Scrape, io::Error>,
    O: FnMut(&[MetricFamily]) -> Result<(), io::Error>,
{
    pub fn in2out<C>(&mut self, conv: C) -> Result<(), io::Error>
    where
        C: ParsedToProto<Error = io::Error>,
    {
        let s: Scrape = (self.input_scrape)()?;
        let converted: Vec<_> = conv.convert(s)?;
        (self.output_proto)(&converted)
    }

    pub fn in2out_default(&mut self) -> Result<(), io::Error> {
        self.in2out(scrape2proto)
    }
}
