use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::{error::Error, fs::File};

#[derive(Debug, Clone, Deserialize)]
pub struct Tick {
    #[serde(rename = "ts")]
    pub ts: DateTime<Utc>,
    pub price: f64,
    pub volume: f64,
}

pub fn load_ticks(path: &str) -> Result<Vec<Tick>, Box<dyn Error>> {
    let file = File::open(path)?;
    let mut rdr = csv::ReaderBuilder::new().has_headers(true).from_reader(file);
    rdr.deserialize().collect::<Result<Vec<_>, _>>().map_err(Into::into)
}
