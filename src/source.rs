use std::io;

use io::BufRead;
use io::BufReader;

use std::fs::File;

use prometheus_parse::Scrape;

pub fn filename2source(filename: String) -> impl FnMut() -> Result<Scrape, io::Error> {
    move || {
        let met_file: File = File::open(&filename)?;
        let br = BufReader::new(met_file);
        let lines = br.lines();
        Scrape::parse(lines)
    }
}
