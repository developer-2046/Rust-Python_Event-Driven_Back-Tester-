use pyo3::prelude::*;
use serde::Deserialize;

mod data;
mod events;
mod strategy;
mod engine;
mod walkforward;
mod grid;

#[derive(Deserialize)]
struct Config {
    fast: usize,
    slow: usize,
    initial_capital: f64,
}

#[pyfunction]
fn backtest(path: &str, cfg_json: &str) -> PyResult<Vec<(i64, f64, f64)>> {
    let cfg: Config = serde_json::from_str(cfg_json)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;

    let ticks = data::load_ticks(path)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;

    let mut eng = engine::Engine::new(cfg.fast, cfg.slow, cfg.initial_capital);
    eng.run(ticks)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

    Ok(eng
        .result()
        .iter()
        .map(|(t, eq, sh)| (t.timestamp_millis(), *eq, *sh))
        .collect())
}

#[pyfunction]
fn walk_forward(
    path: &str,
    train_len: usize,
    test_len: usize,
    fast: usize,
    slow: usize,
) -> PyResult<Vec<(i64, f64, f64)>> {
    let ticks = data::load_ticks(path)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;

    let rows = walkforward::walk_forward(&ticks, train_len, test_len, fast, slow)
        .into_iter()
        .map(|(t, eq, sh)| (t.timestamp_millis(), eq, sh))
        .collect();
    Ok(rows)
}

#[pyfunction]
fn grid_search(
    path: &str,
    fast_min: usize,
    fast_max: usize,
    slow_min: usize,
    slow_max: usize,
) -> PyResult<Vec<(usize, usize, f64)>> {
    let ticks = data::load_ticks(path)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;

    let res = grid::grid_search(
        &ticks,
        fast_min..=fast_max,
        slow_min..=slow_max,
    );
    Ok(res)
}

#[pymodule]
fn rust_core(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(backtest, m)?)?;
    m.add_function(wrap_pyfunction!(walk_forward, m)?)?;
    m.add_function(wrap_pyfunction!(grid_search, m)?)?;
    Ok(())
}
