use crate::{data::Tick, engine::Engine};

pub fn walk_forward(
    ticks: &[Tick],
    train_len: usize,
    test_len: usize,
    fast: usize,
    slow: usize,
) -> Vec<(chrono::DateTime<chrono::Utc>, f64, f64)> {
    let mut out = Vec::new();
    let mut idx = 0;
    while idx + train_len + test_len <= ticks.len() {
        let slice = ticks[idx + train_len .. idx + train_len + test_len].to_vec();
        let mut eng = Engine::new(fast, slow, 100_000.0);
        eng.run(slice).unwrap();
        out.extend_from_slice(eng.result());
        idx += test_len;
    }
    out
}
