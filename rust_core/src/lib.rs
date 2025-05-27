use pyo3::prelude::*;
use serde::Deserialize;

mod data;
mod events;
mod strategy;
mod engine;

#[derive(Deserialize)]
struct Config {
    fast: usize,
    slow: usize,
    initial_capital: f64,
}

#[pyfunction]
fn backtest(path: &str, cfg_json: &str) -> PyResult<pyo3_polars::PyDataFrame> {
    let cfg: Config = serde_json::from_str(cfg_json)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{}", e)))?;

    let ticks = data::load_ticks(path)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(format!("{}", e)))?;

    let mut eng = engine::Engine::new(cfg.fast, cfg.slow, cfg.initial_capital);
    eng.run(ticks).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))?;

    Ok(pyo3_polars::PyDataFrame(eng.equity_df()))
}

#[pymodule]
fn rust_core_py(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(backtest, m)?)?;
    Ok(())
}
