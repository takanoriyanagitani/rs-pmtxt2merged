use std::io;

use std::collections::HashMap;

use chrono::{DateTime, Utc};

use prometheus_parse::Scrape;

use prometheus_parse::Sample;

use prometheus_parse::Labels;
use prometheus_parse::Value;

use prometheus_parse::HistogramCount;
use prometheus_parse::SummaryCount;

pub fn lines2parsed<I>(lines: I) -> Result<Scrape, io::Error>
where
    I: Iterator<Item = Result<String, io::Error>>,
{
    Scrape::parse(lines)
}

pub struct ParsedScrape(pub Scrape);

impl ParsedScrape {
    pub fn as_docs(&self) -> &HashMap<String, String> {
        &self.0.docs
    }

    pub fn as_samples(&self) -> &[Sample] {
        &self.0.samples
    }
}

pub struct ParsedSample(pub Sample);

impl ParsedSample {
    pub fn metric(&self) -> &str {
        &self.0.metric
    }

    pub fn value(&self) -> &Value {
        &self.0.value
    }

    pub fn labels(&self) -> &Labels {
        &self.0.labels
    }

    pub fn timestamp(&self) -> DateTime<Utc> {
        self.0.timestamp
    }
}

pub struct ParsedValue(pub Value);

impl ParsedValue {
    pub fn as_counter(&self) -> Option<f64> {
        match self.0 {
            Value::Counter(c) => Some(c),
            _ => None,
        }
    }

    pub fn as_gauge(&self) -> Option<f64> {
        match self.0 {
            Value::Gauge(g) => Some(g),
            _ => None,
        }
    }

    pub fn as_untyped(&self) -> Option<f64> {
        match self.0 {
            Value::Untyped(f) => Some(f),
            _ => None,
        }
    }

    pub fn as_histogram(&self) -> Option<&[HistogramCount]> {
        match self.0 {
            Value::Histogram(ref v) => Some(v),
            _ => None,
        }
    }

    pub fn as_summary(&self) -> Option<&[SummaryCount]> {
        match self.0 {
            Value::Summary(ref v) => Some(v),
            _ => None,
        }
    }
}

pub struct ParsedHistogram(pub HistogramCount);

impl ParsedHistogram {
    pub fn as_less_than(&self) -> f64 {
        self.0.less_than
    }

    pub fn as_count(&self) -> f64 {
        self.0.count
    }
}

pub struct ParsedSummary(pub SummaryCount);

impl ParsedSummary {
    pub fn as_quantile(&self) -> f64 {
        self.0.quantile
    }

    pub fn as_count(&self) -> f64 {
        self.0.count
    }
}

pub struct ParsedLabels(pub Labels);

impl ParsedLabels {
    pub fn get_by_name(&self, name: &str) -> Option<&str> {
        self.0.get(name)
    }
}

impl ParsedLabels {
    pub fn as_hashmap(&self) -> &HashMap<String, String> {
        &self.0
    }
}

#[cfg(test)]
mod test_parse {
    mod lines2parsed {
        use std::collections::HashMap;

        use prometheus_parse::Labels;
        use prometheus_parse::Sample;
        use prometheus_parse::Scrape;
        use prometheus_parse::Value;

        #[test]
        fn empty() {
            let mtxt: &[u8] = b"";
            let lines = std::io::BufRead::lines(mtxt);
            let rparsed: Result<Scrape, _> = crate::parse::lines2parsed(lines);

            let parsed: Scrape = rparsed.unwrap();
            let docs: &HashMap<String, String> = &parsed.docs;
            let samples: &[Sample] = &parsed.samples;

            assert!(docs.is_empty());
            assert!(samples.is_empty());
        }

        #[test]
        fn simple() {
            let mut mtxt: String = "".into();

            mtxt += "# HELP nvme_power_cycles_total SMART metric power_cycles_total";
            mtxt += "\n";

            mtxt += "# TYPE nvme_power_cycles_total counter";
            mtxt += "\n";

            mtxt += r#"nvme_power_cycles_total{device="nvme0n1"} 28"#;
            mtxt += "\n";

            mtxt += r#"nvme_power_cycles_total{device="nvme1n1"} 38"#;
            mtxt += "\n";

            let lines = std::io::BufRead::lines(mtxt.as_bytes());
            let rparsed: Result<Scrape, _> = crate::parse::lines2parsed(lines);

            let parsed: Scrape = rparsed.unwrap();
            let docs: &HashMap<String, String> = &parsed.docs;
            let samples: &[Sample] = &parsed.samples;

            assert_eq!(2, samples.len());
            assert_eq!(1, docs.len());

            let key0: &str = docs.keys().next().unwrap();
            assert_eq!("nvme_power_cycles_total", key0);

            let val0: &str = docs.get(key0).unwrap();
            assert_eq!("SMART metric power_cycles_total", val0);

            let mut dev2sample: HashMap<String, Sample> = HashMap::default();

            for sample in parsed.samples {
                let lbl: &Labels = &sample.labels;
                let devname: &str = lbl.get("device").unwrap();
                dev2sample.insert(devname.into(), sample);
            }

            let nvme0n1: Sample = dev2sample.remove("nvme0n1").unwrap();
            let nvme1n1: Sample = dev2sample.remove("nvme1n1").unwrap();

            assert_eq!("nvme_power_cycles_total", nvme0n1.metric);
            assert_eq!("nvme_power_cycles_total", nvme1n1.metric);

            let val0: Value = nvme0n1.value;
            let val1: Value = nvme1n1.value;

            assert_eq!(Value::Counter(28.0), val0);
            assert_eq!(Value::Counter(38.0), val1);
        }
    }
}
