// -------------------------------------------------------------------------------------------------
//  Copyright (C) 2015-2026 Nautech Systems Pty Ltd. All rights reserved.
//  https://nautechsystems.io
//
//  Licensed under the GNU Lesser General Public License Version 3.0 (the "License");
//  You may not use this file except in compliance with the License.
//  You may obtain a copy of the License at https://www.gnu.org/licenses/lgpl-3.0.en.html
//
//  Unless required by applicable law or agreed to in writing, software
//  distributed under the License is distributed on an "AS IS" BASIS,
//  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//  See the License for the specific language governing permissions and
//  limitations under the License.
// -------------------------------------------------------------------------------------------------

//! Python bindings for Local L2 Book Engine.

use pyo3::prelude::*;
use crate::book::{LocalL2Book, MarketState, CostEstimatorConfig};
use crate::types::{Side, ImpactResult, LiquidityStats, BookView};

/// Python wrapper for Side enum
#[pyclass(name = "Side", module = "nautilus_trader.core.nautilus_pyo3.stg", from_py_object)]
#[derive(Clone, Copy)]
pub struct PySide(pub Side);

#[pymethods]
impl PySide {
    #[new]
    fn new(side: &str) -> PyResult<Self> {
        match side.to_uppercase().as_str() {
            "BUY" => Ok(Self(Side::Buy)),
            "SELL" => Ok(Self(Side::Sell)),
            _ => Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                format!("Invalid side: {}", side)
            )),
        }
    }
    
    #[staticmethod]
    fn buy() -> Self {
        Self(Side::Buy)
    }
    
    #[staticmethod]
    fn sell() -> Self {
        Self(Side::Sell)
    }
    
    fn __repr__(&self) -> String {
        match self.0 {
            Side::Buy => "Side.BUY".to_string(),
            Side::Sell => "Side.SELL".to_string(),
        }
    }
    
    fn __str__(&self) -> String {
        match self.0 {
            Side::Buy => "BUY".to_string(),
            Side::Sell => "SELL".to_string(),
        }
    }
}

/// Python wrapper for LiquidityStats
#[pyclass(name = "LiquidityStats", module = "nautilus_trader.core.nautilus_pyo3.stg", from_py_object)]
#[derive(Clone)]
pub struct PyLiquidityStats {
    #[pyo3(get)]
    pub bid1: f64,
    #[pyo3(get)]
    pub ask1: f64,
    #[pyo3(get)]
    pub mid: f64,
    #[pyo3(get)]
    pub spread_ticks: f64,
    #[pyo3(get)]
    pub spread_bps: f64,
    #[pyo3(get)]
    pub top_n_total_qty: f64,
    #[pyo3(get)]
    pub gap_ticks_bid: f64,
    #[pyo3(get)]
    pub gap_ticks_ask: f64,
}

impl From<LiquidityStats> for PyLiquidityStats {
    fn from(stats: LiquidityStats) -> Self {
        Self {
            bid1: stats.bid1,
            ask1: stats.ask1,
            mid: stats.mid,
            spread_ticks: stats.spread_ticks,
            spread_bps: stats.spread_bps,
            top_n_total_qty: stats.top_n_total_qty,
            gap_ticks_bid: stats.gap_ticks_bid,
            gap_ticks_ask: stats.gap_ticks_ask,
        }
    }
}

#[pymethods]
impl PyLiquidityStats {
    fn __repr__(&self) -> String {
        format!(
            "LiquidityStats(bid1={:.4}, ask1={:.4}, mid={:.4}, spread_bps={:.2})",
            self.bid1, self.ask1, self.mid, self.spread_bps
        )
    }
}

/// Python wrapper for ImpactResult
#[pyclass(name = "ImpactResult", module = "nautilus_trader.core.nautilus_pyo3.stg", from_py_object)]
#[derive(Clone)]
pub struct PyImpactResult {
    #[pyo3(get)]
    pub ok: bool,
    #[pyo3(get)]
    pub vwap_price: f64,
    #[pyo3(get)]
    pub worst_price: f64,
    #[pyo3(get)]
    pub limit_price: f64,
    #[pyo3(get)]
    pub slip_bps: f64,
    #[pyo3(get)]
    pub used_level: usize,
}

impl From<ImpactResult> for PyImpactResult {
    fn from(result: ImpactResult) -> Self {
        Self {
            ok: result.ok,
            vwap_price: result.vwap_price,
            worst_price: result.worst_price,
            limit_price: result.limit_price,
            slip_bps: result.slip_bps,
            used_level: result.used_level,
        }
    }
}

#[pymethods]
impl PyImpactResult {
    fn __repr__(&self) -> String {
        format!(
            "ImpactResult(ok={}, vwap={:.4}, slip_bps={:.2}, levels={})",
            self.ok, self.vwap_price, self.slip_bps, self.used_level
        )
    }
}

/// Python wrapper for MarketState
#[pyclass(name = "MarketState", module = "nautilus_trader.core.nautilus_pyo3.stg", from_py_object)]
#[derive(Clone)]
pub struct PyMarketState {
    #[pyo3(get, set)]
    pub rvol_bps_1s: f64,
    #[pyo3(get, set)]
    pub bc: f64,
    #[pyo3(get, set)]
    pub lcd: bool,
    #[pyo3(get, set)]
    pub withdraw_score: f64,
    #[pyo3(get, set)]
    pub rate_limit_usage: f64,
    #[pyo3(get, set)]
    pub latency_p95_ms: f64,
}

#[pymethods]
impl PyMarketState {
    #[new]
    #[pyo3(signature = (rvol_bps_1s=10.0, bc=0.95, lcd=false, withdraw_score=0.0, rate_limit_usage=0.0, latency_p95_ms=5.0))]
    fn new(
        rvol_bps_1s: f64,
        bc: f64,
        lcd: bool,
        withdraw_score: f64,
        rate_limit_usage: f64,
        latency_p95_ms: f64,
    ) -> Self {
        Self {
            rvol_bps_1s,
            bc,
            lcd,
            withdraw_score,
            rate_limit_usage,
            latency_p95_ms,
        }
    }
    
    fn __repr__(&self) -> String {
        format!(
            "MarketState(rvol_bps_1s={:.2}, bc={:.3}, lcd={}, latency_p95_ms={:.1})",
            self.rvol_bps_1s, self.bc, self.lcd, self.latency_p95_ms
        )
    }
}

impl From<PyMarketState> for MarketState {
    fn from(state: PyMarketState) -> Self {
        Self {
            rvol_bps_1s: state.rvol_bps_1s,
            bc: state.bc,
            lcd: state.lcd,
            withdraw_score: state.withdraw_score,
            rate_limit_usage: state.rate_limit_usage,
            latency_p95_ms: state.latency_p95_ms,
        }
    }
}

/// Python wrapper for CostEstimatorConfig
#[pyclass(name = "CostEstimatorConfig", module = "nautilus_trader.core.nautilus_pyo3.stg", from_py_object)]
#[derive(Clone)]
pub struct PyCostEstimatorConfig {
    #[pyo3(get, set)]
    pub fee_taker_bps: f64,
    #[pyo3(get, set)]
    pub expected_exec_time_ms: f64,
    #[pyo3(get, set)]
    pub drift_rate_per_rvol: f64,
    #[pyo3(get, set)]
    pub bc_penalty_factor: f64,
    #[pyo3(get, set)]
    pub lcd_penalty_bps: f64,
    #[pyo3(get, set)]
    pub withdraw_penalty_factor: f64,
    #[pyo3(get, set)]
    pub rate_limit_penalty_factor: f64,
    #[pyo3(get, set)]
    pub latency_drift_rate_per_ms: f64,
    #[pyo3(get, set)]
    pub insufficient_liquidity_penalty_bps: f64,
    #[pyo3(get, set)]
    pub extrapolation_ratio_limit: f64,
    #[pyo3(get, set)]
    pub no_liquidity_penalty_bps: f64,
}

#[pymethods]
impl PyCostEstimatorConfig {
    #[new]
    #[pyo3(signature = (
        fee_taker_bps=5.0,
        expected_exec_time_ms=100.0,
        drift_rate_per_rvol=0.5,
        bc_penalty_factor=20.0,
        lcd_penalty_bps=50.0,
        withdraw_penalty_factor=10.0,
        rate_limit_penalty_factor=30.0,
        latency_drift_rate_per_ms=0.1,
        insufficient_liquidity_penalty_bps=50.0,
        extrapolation_ratio_limit=3.0,
        no_liquidity_penalty_bps=200.0
    ))]
    fn new(
        fee_taker_bps: f64,
        expected_exec_time_ms: f64,
        drift_rate_per_rvol: f64,
        bc_penalty_factor: f64,
        lcd_penalty_bps: f64,
        withdraw_penalty_factor: f64,
        rate_limit_penalty_factor: f64,
        latency_drift_rate_per_ms: f64,
        insufficient_liquidity_penalty_bps: f64,
        extrapolation_ratio_limit: f64,
        no_liquidity_penalty_bps: f64,
    ) -> Self {
        Self {
            fee_taker_bps,
            expected_exec_time_ms,
            drift_rate_per_rvol,
            bc_penalty_factor,
            lcd_penalty_bps,
            withdraw_penalty_factor,
            rate_limit_penalty_factor,
            latency_drift_rate_per_ms,
            insufficient_liquidity_penalty_bps,
            extrapolation_ratio_limit,
            no_liquidity_penalty_bps,
        }
    }
    
    fn __repr__(&self) -> String {
        format!(
            "CostEstimatorConfig(fee_taker_bps={:.1}, exec_time_ms={:.1})",
            self.fee_taker_bps, self.expected_exec_time_ms
        )
    }
}

impl From<PyCostEstimatorConfig> for CostEstimatorConfig {
    fn from(config: PyCostEstimatorConfig) -> Self {
        Self {
            fee_taker_bps: config.fee_taker_bps,
            expected_exec_time_ms: config.expected_exec_time_ms,
            drift_rate_per_rvol: config.drift_rate_per_rvol,
            bc_penalty_factor: config.bc_penalty_factor,
            lcd_penalty_bps: config.lcd_penalty_bps,
            withdraw_penalty_factor: config.withdraw_penalty_factor,
            rate_limit_penalty_factor: config.rate_limit_penalty_factor,
            latency_drift_rate_per_ms: config.latency_drift_rate_per_ms,
            insufficient_liquidity_penalty_bps: config.insufficient_liquidity_penalty_bps,
            extrapolation_ratio_limit: config.extrapolation_ratio_limit,
            no_liquidity_penalty_bps: config.no_liquidity_penalty_bps,
        }
    }
}

/// Python wrapper for LocalL2Book
#[pyclass(name = "LocalL2Book", module = "nautilus_trader.core.nautilus_pyo3.stg")]
pub struct PyLocalL2Book {
    inner: LocalL2Book,
}

#[pymethods]
impl PyLocalL2Book {
    /// Create a new LocalL2Book
    ///
    /// Parameters
    /// ----------
    /// top_k : int
    ///     Maximum number of levels to maintain (default: 100)
    /// top_n : int
    ///     Number of top levels for liquidity stats (default: 10)
    /// tick_size : float
    ///     Minimum price increment (default: 0.01)
    /// cost_config : CostEstimatorConfig, optional
    ///     Cost estimation configuration
    ///
    /// Returns
    /// -------
    /// LocalL2Book
    ///
    #[new]
    #[pyo3(signature = (top_k=100, top_n=10, tick_size=0.01, cost_config=None))]
    fn new(
        top_k: usize,
        top_n: usize,
        tick_size: f64,
        cost_config: Option<PyCostEstimatorConfig>,
    ) -> Self {
        let inner = if let Some(config) = cost_config {
            LocalL2Book::with_full_config(top_k, top_n, tick_size, config.into())
        } else {
            LocalL2Book::with_full_config(
                top_k,
                top_n,
                tick_size,
                CostEstimatorConfig::default(),
            )
        };
        Self { inner }
    }
    
    /// Update order book with a price level
    ///
    /// Parameters
    /// ----------
    /// side : Side
    ///     BUY or SELL side
    /// price : float
    ///     Price level
    /// size : float
    ///     Quantity at this level (0 to delete)
    ///
    fn update(&mut self, side: &PySide, price: f64, size: f64) {
        self.inner.update(side.0, price, size);
    }
    
    /// Get best bid/ask and liquidity statistics
    ///
    /// Returns
    /// -------
    /// LiquidityStats
    ///
    fn best(&self) -> PyLiquidityStats {
        self.inner.best().into()
    }
    
    /// Calculate impact price for a given quantity
    ///
    /// Parameters
    /// ----------
    /// side : Side
    ///     Direction of trade (BUY or SELL)
    /// qty : float
    ///     Quantity to trade
    /// cushion_ticks : int
    ///     Additional ticks for limit price (default: 0)
    ///
    /// Returns
    /// -------
    /// ImpactResult
    ///
    #[pyo3(signature = (side, qty, cushion_ticks=0))]
    fn impact(&self, side: &PySide, qty: f64, cushion_ticks: u32) -> PyImpactResult {
        self.inner.impact(side.0, qty, cushion_ticks).into()
    }
    
    /// Get top K levels
    ///
    /// Parameters
    /// ----------
    /// side : Side
    ///     BUY or SELL side
    /// k : int
    ///     Number of levels to return
    ///
    /// Returns
    /// -------
    /// tuple[list[float], list[float]]
    ///     (prices, sizes)
    ///
    fn top_k(&self, side: &PySide, k: usize) -> (Vec<f64>, Vec<f64>) {
        self.inner.top_k(side.0, k)
    }
    
    /// Calculate microprice
    ///
    /// Returns
    /// -------
    /// float or None
    ///     Microprice if valid book, None otherwise
    ///
    fn microprice(&self) -> Option<f64> {
        self.inner.microprice()
    }
    
    /// Calculate microprice deviation from mid in bps
    ///
    /// Returns
    /// -------
    /// float or None
    ///     Deviation in bps if valid book, None otherwise
    ///
    fn microprice_deviation_bps(&self) -> Option<f64> {
        self.inner.microprice_deviation_bps()
    }
    
    /// Estimate bailout cost in bps
    ///
    /// Parameters
    /// ----------
    /// qty : float
    ///     Quantity to liquidate
    /// side : Side
    ///     Direction of trade (BUY or SELL)
    /// market_state : MarketState
    ///     Current market conditions
    ///
    /// Returns
    /// -------
    /// float
    ///     Estimated cost in basis points
    ///
    fn bailout_cost_bps_est(&self, qty: f64, side: &PySide, market_state: &PyMarketState) -> f64 {
        let state: MarketState = market_state.clone().into();
        self.inner.bailout_cost_bps_est(qty, side.0, &state)
    }
    
    /// Get current cost estimator config
    ///
    /// Returns
    /// -------
    /// CostEstimatorConfig
    ///
    fn cost_config(&self) -> PyCostEstimatorConfig {
        let config = self.inner.cost_config();
        PyCostEstimatorConfig {
            fee_taker_bps: config.fee_taker_bps,
            expected_exec_time_ms: config.expected_exec_time_ms,
            drift_rate_per_rvol: config.drift_rate_per_rvol,
            bc_penalty_factor: config.bc_penalty_factor,
            lcd_penalty_bps: config.lcd_penalty_bps,
            withdraw_penalty_factor: config.withdraw_penalty_factor,
            rate_limit_penalty_factor: config.rate_limit_penalty_factor,
            latency_drift_rate_per_ms: config.latency_drift_rate_per_ms,
            insufficient_liquidity_penalty_bps: config.insufficient_liquidity_penalty_bps,
            extrapolation_ratio_limit: config.extrapolation_ratio_limit,
            no_liquidity_penalty_bps: config.no_liquidity_penalty_bps,
        }
    }
    
    /// Set cost estimator config
    ///
    /// Parameters
    /// ----------
    /// config : CostEstimatorConfig
    ///     New configuration
    ///
    fn set_cost_config(&mut self, config: PyCostEstimatorConfig) {
        self.inner.set_cost_config(config.into());
    }
    
    fn __repr__(&self) -> String {
        let stats = self.inner.best();
        format!(
            "LocalL2Book(bid1={:.4}, ask1={:.4}, mid={:.4})",
            stats.bid1, stats.ask1, stats.mid
        )
    }
}
