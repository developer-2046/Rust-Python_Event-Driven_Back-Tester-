use crate::data::Tick;
use crate::events::{Event, FillEvent, OrderEvent};
use crate::strategy::SmaCross;
use anyhow::Result;
use std::collections::VecDeque;

const ES_MULT: f64 = 50.0;
const ROLL: usize = 2_000;        // rolling Sharpe window

pub struct Engine {
    queue: VecDeque<Event>,
    strategy: SmaCross,
    equity: f64,
    position: i32,
    equity_curve: Vec<(chrono::DateTime<chrono::Utc>, f64, f64)>,
    rolling_buf: Vec<f64>,
}

impl Engine {
    pub fn new(fast: usize, slow: usize, start_cash: f64) -> Self {
        Self {
            queue: VecDeque::new(),
            strategy: SmaCross::new(fast, slow),
            equity: start_cash,
            position: 0,
            equity_curve: Vec::new(),
            rolling_buf: Vec::with_capacity(ROLL + 1),
        }
    }

    pub fn run(&mut self, ticks: Vec<Tick>) -> Result<()> {
        for tick in ticks {
            self.queue.push_back(Event::Market(tick.clone()));
            while let Some(ev) = self.queue.pop_front() {
                match ev {
                    Event::Market(t) => self.handle_market(t),
                    Event::Signal(s) => self.handle_signal(s),
                    Event::Order(o)  => self.handle_order(o, tick.price),
                    Event::Fill(f)   => self.handle_fill(f),
                }
            }
        }
        Ok(())
    }

    fn handle_market(&mut self, tick: Tick) {
        if let Some(sig) = self.strategy.on_tick(&tick) {
            self.queue.push_back(sig);
        }
        let mv = self.position as f64 * tick.price * ES_MULT;
        self.push_equity(tick.ts, self.equity + mv);
    }

    fn handle_signal(&mut self, sig: crate::events::SignalEvent) {
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
        let sign = if fill.long { 1 } else { -1 };
        self.position += sign * fill.size;
        self.equity -= sign as f64 * fill.size as f64 * fill.price * ES_MULT;
        self.equity -= fill.commission;
    }

    fn push_equity(&mut self, ts: chrono::DateTime<chrono::Utc>, eq: f64) {
        if let Some(_) = self.rolling_buf.last() {
            self.rolling_buf.push(eq);
            if self.rolling_buf.len() > ROLL + 1 {
                self.rolling_buf.remove(0);
            }
        } else {
            self.rolling_buf.push(eq);
        }

        let sharpe = if self.rolling_buf.len() > 2 {
            let rets: Vec<f64> = self
                .rolling_buf
                .windows(2)
                .map(|w| w[1] / w[0] - 1.0)
                .collect();
            let mean = rets.iter().sum::<f64>() / rets.len() as f64;
            let var =
                rets.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / rets.len() as f64;
            if var > 0.0 {
                mean / var.sqrt() * (252_f64).sqrt()
            } else {
                0.0
            }
        } else {
            0.0
        };

        self.equity_curve.push((ts, eq, sharpe));
    }

    pub fn result(&self) -> &[(chrono::DateTime<chrono::Utc>, f64, f64)] {
        &self.equity_curve
    }
}
