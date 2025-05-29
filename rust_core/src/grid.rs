use crate::{data::Tick, engine::Engine};
use rayon::prelude::*;

pub fn grid_search(
    ticks: &[Tick],
    fast_range: std::ops::RangeInclusive<usize>,
    slow_range: std::ops::RangeInclusive<usize>,
) -> Vec<(usize, usize, f64)> {
    fast_range
        .into_par_iter()
        .flat_map_iter(|f| slow_range.clone().map(move |s| (f, s)))
        .filter(|(f, s)| f < s) // logical constraint
        .map(|(f, s)| {
            let mut eng = Engine::new(f, s, 100_000.0);
            eng.run(ticks.to_vec()).unwrap();
            let sharpe = eng.result().last().map(|t| t.2).unwrap_or(0.0);
            (f, s, sharpe)
        })
        .collect()
}
