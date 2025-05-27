use polars::prelude::*;
use chrono::{DateTime, Utc};
use anyhow::Result;
use std::fs::File;

#[derive(Debug, Clone)]
pub struct Tick {
    pub ts: DateTime<Utc>,
    pub price: f64,
    pub volume: f64,
}

pub fn load_ticks(path: &str) -> Result<Vec<Tick>> {
    let file = File::open(path)?;
let df = CsvReader::new(file)
    .has_header(true)
    .with_try_parse_dates(true)
    .finish()?;


    let ts_ca = df.column("timestamp")?.datetime()?;
    let price = df.column("price")?.f64()?;
    let vol   = df.column("volume")?.f64()?;

    let ticks = (0..df.height())
        .map(|i| Tick {
            ts:  DateTime::<Utc>::from(ts_ca.get(i).unwrap()),
            price: price.get(i).unwrap(),
            volume: vol.get(i).unwrap(),
        })
        .collect();

    Ok(ticks)
}
