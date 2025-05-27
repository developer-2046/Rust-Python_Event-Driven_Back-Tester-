use crate::events::{Event, OrderEvent, FillEvent};
use crate::data::Tick;
use crate::strategy::SmaCross;
use anyhow::Result;
use std::collections::VecDeque;

pub struct Engine {
    queue: VecDeque<Event>,
    strategy: SmaCross,
    equity: f64,
    position: i32,
    equity_curve: Vec<(chrono::DateTime<chrono::Utc>, f64)>,
    
}

impl Engine {
    pub fn new(fast: usize, slow: usize, equity: f64) -> Self {
        Self {
            queue: VecDeque::new(),
            strategy: SmaCross::new(fast, slow),
            equity,
            position: 0,
            equity_curve: Vec::new(),
        }
    }

    // ========================================== core loop
    pub fn run(&mut self, ticks: Vec<Tick>) -> Result<()> {
        for tick in ticks {
            self.queue.push_back(Event::Market(tick.clone()));

            while let Some(ev) = self.queue.pop_front() {
                match ev {
                    Event::Market(t) => self.handle_market(t),
                    Event::Signal(s) => self.handle_signal(s, tick.price),
                    Event::Order(o)  => self.handle_order(o, tick.price),
                    Event::Fill(f)   => self.handle_fill(f),
                }
            }
        }
        Ok(())
    }

    // -------------- handlers --------------
    fn handle_market(&mut self, tick: Tick) {
        if let Some(sig) = self.strategy.on_tick(&tick) {
            self.queue.push_back(sig);
        }
        let mv = self.position as f64 * tick.price * 50.0; // ES multiplier
        self.equity_curve.push((tick.ts, self.equity + mv));
    }

    fn handle_signal(&mut self, sig: crate::events::SignalEvent, price: f64) {
        let desired = if sig.long { 1 } else { 0 };
        let diff = desired - self.position;
        if diff != 0 {
            self.queue.push_back(Event::Order(OrderEvent {
                ts: sig.ts,
                long: diff > 0,
                size: diff.abs(),
            }));
        }
    }

    fn handle_order(&mut self, ord: OrderEvent, price: f64) {
        self.queue.push_back(Event::Fill(FillEvent {
            ts: ord.ts,
            long: ord.long,
            size: ord.size,
            price,
            commission: 2.5,
        }));
    }

    fn handle_fill(&mut self, fill: FillEvent) {
        let signed = if fill.long { 1 } else { -1 };
        self.position += signed * fill.size;
        self.equity -= signed as f64 * fill.size as f64 * fill.price * 50.0;
        self.equity -= fill.commission;
    }

    // -------------- output --------------
pub fn equity_df(&self) -> polars::prelude::DataFrame {
    use polars::prelude::*;
    let ts_i64 : Vec<i64> = self.equity_curve
        .iter()
        .map(|(t, _)| t.timestamp_millis())
        .collect();
    let eq_vec : Vec<f64> = self.equity_curve
        .iter()
        .map(|(_, e)| *e)
        .collect();
    
    let ts_series = Series::new("timestamp", ts_i64);
    let eq_series = Series::new("equity",    eq_vec);
    let df = DataFrame::new(vec![ts_series, eq_series]).unwrap();


    DataFrame::new(vec![ts_series, eq_series]).unwrap()
}
}