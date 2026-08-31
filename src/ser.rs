use std::io;

use io::Write;

use prometheus::TextEncoder;

use prometheus::proto::MetricFamily;

use prometheus::proto::Metric;
use prometheus::proto::MetricType;

use prometheus::proto::LabelPair;

use prometheus::proto::Counter;
use prometheus::proto::Gauge;

#[derive(Default)]
pub struct MetricToString(pub TextEncoder);

impl MetricToString {
    pub fn met2str(&self, met: &[MetricFamily], buf: &mut String) -> Result<(), io::Error> {
        self.0.encode_utf8(met, buf).map_err(io::Error::other)
    }
}

pub fn wtr2met_consumer<W>(mut wtr: W) -> impl FnMut(&[MetricFamily]) -> Result<(), io::Error>
where
    W: Write,
{
    let mut buf: String = String::default();
    let enc = MetricToString::default();
    move |m: &[_]| {
        buf.clear();
        enc.met2str(m, &mut buf)?;
        let b: &[u8] = buf.as_bytes();
        wtr.write_all(b)?;
        wtr.flush()
    }
}

pub struct BasicLabel {
    pub name: String,
    pub value: String,
}

impl BasicLabel {
    pub fn convert(b: Vec<Self>) -> Vec<LabelPair> {
        b.into_iter().map(|i| i.into()).collect()
    }
}

impl From<BasicLabel> for LabelPair {
    fn from(b: BasicLabel) -> Self {
        let mut lp: LabelPair = LabelPair::default();
        lp.set_name(b.name);
        lp.set_value(b.value);
        lp
    }
}

pub struct BasicGauge(pub f64);

impl From<BasicGauge> for Gauge {
    fn from(b: BasicGauge) -> Self {
        let mut g: Gauge = Gauge::default();
        g.set_value(b.0);
        g
    }
}

pub struct BasicCounter(pub f64);

impl From<BasicCounter> for Counter {
    fn from(b: BasicCounter) -> Self {
        let mut c: Counter = Counter::default();
        c.set_value(b.0);
        c
    }
}

pub struct BasicMetric {
    labels: Vec<BasicLabel>,
    gauge: Option<BasicGauge>,
    counter: Option<BasicCounter>,
}

impl BasicMetric {
    pub fn convert(b: Vec<Self>) -> Vec<Metric> {
        b.into_iter().map(|i| i.into()).collect()
    }
}

impl BasicMetric {
    pub fn from_counter(labels: Vec<BasicLabel>, c: BasicCounter) -> Self {
        Self {
            labels,
            gauge: None,
            counter: Some(c),
        }
    }

    pub fn from_gauge(labels: Vec<BasicLabel>, g: BasicGauge) -> Self {
        Self {
            labels,
            gauge: Some(g),
            counter: None,
        }
    }
}

impl From<BasicMetric> for Metric {
    fn from(b: BasicMetric) -> Self {
        let mut m: Metric = Metric::default();

        if let Some(gauge) = b.gauge {
            m.set_gauge(gauge.into());
        }

        if let Some(counter) = b.counter {
            m.set_counter(counter.into());
        }

        let labels: Vec<LabelPair> = BasicLabel::convert(b.labels);
        m.set_label(labels);

        m
    }
}

pub struct BasicFamily {
    pub ftyp: MetricType,
    pub help: String,
    pub met: Vec<BasicMetric>,
    pub name: String,
}

impl BasicFamily {
    pub fn convert(b: Vec<Self>) -> Vec<MetricFamily> {
        b.into_iter().map(|i| i.into()).collect()
    }
}

impl From<BasicFamily> for MetricFamily {
    fn from(b: BasicFamily) -> Self {
        let mut f: MetricFamily = MetricFamily::default();

        f.set_field_type(b.ftyp);
        f.set_help(b.help);
        f.set_metric(BasicMetric::convert(b.met));
        f.set_name(b.name);

        f
    }
}

#[cfg(test)]
mod test_ser {
    mod metric_to_string {
        mod met2str {

            use prometheus::proto::MetricFamily;
            use prometheus::proto::MetricType;

            use crate::ser::MetricToString;

            use crate::ser::BasicCounter;
            use crate::ser::BasicFamily;
            use crate::ser::BasicGauge;
            use crate::ser::BasicLabel;
            use crate::ser::BasicMetric;

            #[test]
            fn empty() {
                let emet: Vec<MetricFamily> = vec![];
                let mut buf: String = "".into();

                let ser: MetricToString = MetricToString::default();
                ser.met2str(&emet, &mut buf).unwrap();

                assert!(buf.is_empty());
            }

            #[test]
            fn simple() {
                let fam0 = BasicFamily {
                    ftyp: MetricType::COUNTER,
                    help: "Seconds the CPUs spent in each mode.".into(),
                    met: vec![
                        BasicMetric::from_counter(
                            vec![
                                BasicLabel {
                                    name: "cpu".into(),
                                    value: "0".into(),
                                },
                                BasicLabel {
                                    name: "mode".into(),
                                    value: "iowait".into(),
                                },
                            ],
                            BasicCounter(28446.13),
                        ),
                        BasicMetric::from_counter(
                            vec![
                                BasicLabel {
                                    name: "cpu".into(),
                                    value: "0".into(),
                                },
                                BasicLabel {
                                    name: "mode".into(),
                                    value: "system".into(),
                                },
                            ],
                            BasicCounter(107078.8),
                        ),
                        BasicMetric::from_counter(
                            vec![
                                BasicLabel {
                                    name: "cpu".into(),
                                    value: "1".into(),
                                },
                                BasicLabel {
                                    name: "mode".into(),
                                    value: "iowait".into(),
                                },
                            ],
                            BasicCounter(38446.13),
                        ),
                        BasicMetric::from_counter(
                            vec![
                                BasicLabel {
                                    name: "cpu".into(),
                                    value: "1".into(),
                                },
                                BasicLabel {
                                    name: "mode".into(),
                                    value: "system".into(),
                                },
                            ],
                            BasicCounter(207078.8),
                        ),
                    ],
                    name: "node_cpu_seconds_total".into(),
                };

                let fam1 = BasicFamily {
                    ftyp: MetricType::GAUGE,
                    help: "mtu_bytes value of /sys/class/net/<iface>".into(),
                    met: vec![
                        BasicMetric::from_gauge(
                            vec![BasicLabel {
                                name: "device".into(),
                                value: "lo".into(),
                            }],
                            BasicGauge(65536.0),
                        ),
                        BasicMetric::from_gauge(
                            vec![BasicLabel {
                                name: "device".into(),
                                value: "virbr0".into(),
                            }],
                            BasicGauge(1500.0),
                        ),
                    ],
                    name: "node_network_mtu_bytes".into(),
                };

                let emet: Vec<MetricFamily> = BasicFamily::convert(vec![fam0, fam1]);
                let mut buf: String = "".into();

                let ser: MetricToString = MetricToString::default();
                ser.met2str(&emet, &mut buf).unwrap();

                assert!(!buf.is_empty());
            }
        }
    }
}
