use std::io;
use std::process::ExitCode;

use io::BufWriter;
use io::Write;

use rs_pmtxt2merged::conv::InOut;
use rs_pmtxt2merged::merge::state_next2merged;
use rs_pmtxt2merged::ser::wtr2met_consumer;
use rs_pmtxt2merged::source::filename2source;

fn envkey2val(key: &'static str) -> impl Fn() -> String {
    move || std::env::var(String::from(key)).unwrap_or_default()
}

fn state_filename() -> impl Fn() -> String {
    envkey2val("ENV_STATE_MET_TXT_FILENAME")
}

fn next_filename() -> impl Fn() -> String {
    envkey2val("ENV_NEXT_MET_TXT_FILENAME")
}

fn sub() -> Result<(), io::Error> {
    let sfilename: String = state_filename()();
    let nfilename: String = next_filename()();

    let src_s = filename2source(sfilename); // () -> Scrape
    let src_n = filename2source(nfilename); // () -> Scrape

    let src_m = state_next2merged(src_s, src_n); // () -> Scrape

    let o = io::stdout();
    let mut ol = o.lock();
    {
        // (&[MetricFamily]) -> ()
        let met_consumer = wtr2met_consumer(BufWriter::new(&mut ol));

        let mut inout = InOut {
            input_scrape: src_m,
            output_proto: met_consumer,
        };

        inout.in2out_default()?;
    }
    ol.flush()
}

fn main() -> ExitCode {
    sub().map(|_| ExitCode::SUCCESS).unwrap_or_else(|e| {
        eprintln!("{e}");
        ExitCode::FAILURE
    })
}
