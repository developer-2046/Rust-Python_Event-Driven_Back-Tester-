use crate::data::Tick;
use crate::events::{Event, SignalEvent};
use std::collections::VecDeque;

pub struct SmaCross {
    fast: usize,
    slow: usize,
    prices: VecDeque<f64>,
    sum_fast: f64,
    sum_slow: f64,
    in_market: bool,
}

impl SmaCross {
    pub fn new(fast: usize, slow: usize) -> Self {
        Self {
            fast,
            slow,
            prices: VecDeque::with_capacity(slow),
            sum_fast: 0.0,
            sum_slow: 0.0,
            in_market: false,
        }
    }

    pub fn on_tick(&mut self, tick: &Tick) -> Option<Event> {
        self.prices.push_back(tick.price);
        if self.prices.len() > self.slow {
            let removed = self.prices.pop_front().unwrap();
            if self.prices.len() >= self.fast {
                self.sum_fast -= removed;
            }
            self.sum_slow -= removed;
        }
        if self.prices.len() >= self.fast {
            self.sum_fast += tick.price;
        }
        self.sum_slow += tick.price;

        if self.prices.len() < self.slow {
            return None;
        }

        let sma_fast = self.sum_fast / self.fast as f64;
        let sma_slow = self.sum_slow / self.slow as f64;

        match (sma_fast > sma_slow, self.in_market) {
            (true, false) => {
                self.in_market = true;
                Some(Event::Signal(SignalEvent { ts: tick.ts, long: true }))
            }
            (false, true) => {
                self.in_market = false;
                Some(Event::Signal(SignalEvent { ts: tick.ts, long: false }))
            }
            _ => None,
        }
    }
}
