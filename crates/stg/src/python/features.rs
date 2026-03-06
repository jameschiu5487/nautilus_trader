use pyo3::prelude::*;
use crate::features::{OfiCalculator, VolatilityCalculator, LiquidityCalculator, JumpCalculator};
use super::book::PySide;

/// Python wrapper for OfiCalculator
#[pyclass(name = "OfiCalculator", module = "nautilus_trader.core.nautilus_pyo3.stg")]
pub struct PyOfiCalculator {
    inner: OfiCalculator,
}

#[pymethods]
impl PyOfiCalculator {
    /// Create a new OFI calculator with configurable window
    ///
    /// Parameters
    /// ----------
    /// window_ms : int
    ///     Time window size in milliseconds (e.g., 200, 300, 500, 1000)
    ///
    /// Returns
    /// -------
    /// OfiCalculator
    ///
    /// Examples
    /// --------
    /// >>> ofi_200ms = OfiCalculator(200)   # 200ms window
    /// >>> ofi_300ms = OfiCalculator(300)   # 300ms window
    /// >>> ofi_1s = OfiCalculator(1000)     # 1s window
    ///
    #[new]
    fn new(window_ms: u64) -> Self {
        Self {
            inner: OfiCalculator::new(window_ms),
        }
    }
    
    /// Get the window size in milliseconds
    ///
    /// Returns
    /// -------
    /// int
    ///     Window size in milliseconds
    ///
    #[getter]
    fn window_ms(&self) -> u64 {
        self.inner.window_ms()
    }
    
    /// Add an order event and update OFI
    ///
    /// Parameters
    /// ----------
    /// timestamp_ns : int
    ///     Event timestamp in nanoseconds
    /// side : Side
    ///     Order side (BUY or SELL)
    /// price : float
    ///     Order price
    /// size : float
    ///     Order size
    /// is_cancel : bool
    ///     Whether this is a cancellation
    ///
    fn add_order_event(
        &mut self,
        timestamp_ns: u64,
        side: &PySide,
        price: f64,
        size: f64,
        is_cancel: bool,
    ) {
        self.inner.add_order_event(timestamp_ns, side.0, price, size, is_cancel);
    }
    
    /// Get current Order Flow Imbalance
    ///
    /// Returns
    /// -------
    /// float
    ///     OFI value (positive = buy pressure, negative = sell pressure)
    ///
    fn ofi(&self) -> f64 {
        self.inner.ofi()
    }
    
    /// Get current cancel ratio
    ///
    /// Returns
    /// -------
    /// float
    ///     Ratio of cancelled orders (0.0 to 1.0)
    ///
    fn cancel_ratio(&self) -> f64 {
        self.inner.cancel_ratio()
    }
    
    fn __repr__(&self) -> String {
        format!(
            "OfiCalculator(window_ms={}, ofi={:.2}, cancel_ratio={:.3})",
            self.window_ms(),
            self.ofi(),
            self.cancel_ratio()
        )
    }
}

/// Python wrapper for VolatilityCalculator
#[pyclass(name = "VolatilityCalculator", module = "nautilus_trader.core.nautilus_pyo3.stg")]
pub struct PyVolatilityCalculator {
    inner: VolatilityCalculator,
}

#[pymethods]
impl PyVolatilityCalculator {
    /// Create a new volatility calculator with configurable window
    ///
    /// Parameters
    /// ----------
    /// window_ms : int
    ///     Time window size in milliseconds (e.g., 200, 300, 500, 1000)
    ///
    /// Returns
    /// -------
    /// VolatilityCalculator
    ///
    /// Examples
    /// --------
    /// >>> vol_200ms = VolatilityCalculator(200)   # 200ms window
    /// >>> vol_500ms = VolatilityCalculator(500)   # 500ms window
    /// >>> vol_1s = VolatilityCalculator(1000)     # 1s window
    ///
    #[new]
    fn new(window_ms: u64) -> Self {
        Self {
            inner: VolatilityCalculator::new(window_ms),
        }
    }
    
    /// Get the window size in milliseconds
    ///
    /// Returns
    /// -------
    /// int
    ///     Window size in milliseconds
    ///
    #[getter]
    fn window_ms(&self) -> u64 {
        self.inner.window_ms()
    }
    
    /// Update with a new price tick
    ///
    /// Parameters
    /// ----------
    /// timestamp_ns : int
    ///     Price timestamp in nanoseconds
    /// price : float
    ///     Price value
    ///
    fn update(&mut self, timestamp_ns: u64, price: f64) {
        self.inner.update(timestamp_ns, price);
    }
    
    /// Get realized volatility in basis points
    ///
    /// Returns
    /// -------
    /// float
    ///     Annualized volatility in bps
    ///
    fn rvol_bps(&self) -> f64 {
        self.inner.rvol_bps()
    }
    
    fn __repr__(&self) -> String {
        format!(
            "VolatilityCalculator(window_ms={}, rvol_bps={:.2})",
            self.window_ms(),
            self.rvol_bps()
        )
    }
}

/// Python wrapper for LiquidityCalculator
#[pyclass(name = "LiquidityCalculator", module = "nautilus_trader.core.nautilus_pyo3.stg")]
pub struct PyLiquidityCalculator {
    inner: LiquidityCalculator,
}

#[pymethods]
impl PyLiquidityCalculator {
    /// Create a new liquidity calculator with configurable window
    ///
    /// Parameters
    /// ----------
    /// window_ms : int
    ///     Time window size in milliseconds (e.g., 500, 1000, 2000)
    ///
    /// Returns
    /// -------
    /// LiquidityCalculator
    ///
    /// Examples
    /// --------
    /// >>> liq_1s = LiquidityCalculator(1000)   # 1s window
    /// >>> liq_2s = LiquidityCalculator(2000)   # 2s window
    ///
    #[new]
    fn new(window_ms: u64) -> Self {
        Self {
            inner: LiquidityCalculator::new(window_ms),
        }
    }
    
    /// Get the window size in milliseconds
    ///
    /// Returns
    /// -------
    /// int
    ///     Window size in milliseconds
    ///
    #[getter]
    fn window_ms(&self) -> u64 {
        self.inner.window_ms()
    }
    
    /// Add a trade and update statistics
    ///
    /// Parameters
    /// ----------
    /// timestamp_ns : int
    ///     Trade timestamp in nanoseconds
    /// price : float
    ///     Trade price
    /// size : float
    ///     Trade size
    /// side : Side
    ///     Trade side (BUY or SELL)
    ///
    fn add_trade(&mut self, timestamp_ns: u64, price: f64, size: f64, side: &PySide) {
        self.inner.add_trade(timestamp_ns, price, size, side.0);
    }
    
    /// Get average trade quantity
    ///
    /// Returns
    /// -------
    /// float
    ///     Average trade size in the window
    ///
    fn avg_trade_qty(&self) -> f64 {
        self.inner.avg_trade_qty()
    }
    
    /// Calculate liquidity score
    ///
    /// Parameters
    /// ----------
    /// top_n_total_qty : float
    ///     Total quantity in top N levels
    ///
    /// Returns
    /// -------
    /// float
    ///     Liquidity score (higher = better liquidity)
    ///
    fn liquidity_score(&self, top_n_total_qty: f64) -> f64 {
        self.inner.liquidity_score(top_n_total_qty)
    }
    
    fn __repr__(&self) -> String {
        format!(
            "LiquidityCalculator(window_ms={}, avg_trade_qty={:.2})",
            self.window_ms(),
            self.avg_trade_qty()
        )
    }
}

/// Python wrapper for JumpCalculator
#[pyclass(name = "JumpCalculator", module = "nautilus_trader.core.nautilus_pyo3.stg")]
pub struct PyJumpCalculator {
    inner: JumpCalculator,
}

#[pymethods]
impl PyJumpCalculator {
    /// Create a new jump calculator with configurable window and threshold
    ///
    /// Parameters
    /// ----------
    /// window_ms : int
    ///     Time window size in milliseconds (e.g., 500, 1000, 2000)
    /// threshold_bps : float
    ///     Jump threshold in basis points (default: 50)
    ///
    /// Returns
    /// -------
    /// JumpCalculator
    ///
    /// Examples
    /// --------
    /// >>> jump_1s_50bps = JumpCalculator(1000, 50.0)   # 1s window, 50 bps threshold
    /// >>> jump_2s_100bps = JumpCalculator(2000, 100.0) # 2s window, 100 bps threshold
    ///
    #[new]
    #[pyo3(signature = (window_ms, threshold_bps=50.0))]
    fn new(window_ms: u64, threshold_bps: f64) -> Self {
        Self {
            inner: JumpCalculator::new(window_ms, threshold_bps),
        }
    }
    
    /// Get the window size in milliseconds
    ///
    /// Returns
    /// -------
    /// int
    ///     Window size in milliseconds
    ///
    #[getter]
    fn window_ms(&self) -> u64 {
        self.inner.window_ms()
    }
    
    /// Update with a new price tick
    ///
    /// Parameters
    /// ----------
    /// timestamp_ns : int
    ///     Price timestamp in nanoseconds
    /// price : float
    ///     Price value
    ///
    fn update(&mut self, timestamp_ns: u64, price: f64) {
        self.inner.update(timestamp_ns, price);
    }
    
    /// Get number of price jumps in the window
    ///
    /// Returns
    /// -------
    /// int
    ///     Number of jumps exceeding threshold
    ///
    fn jump_count(&self) -> usize {
        self.inner.jump_count()
    }
    
    /// Get maximum price move in the window
    ///
    /// Returns
    /// -------
    /// float
    ///     Maximum move in basis points
    ///
    fn max_move_bps(&self) -> f64 {
        self.inner.max_move_bps()
    }
    
    fn __repr__(&self) -> String {
        format!(
            "JumpCalculator(window_ms={}, jump_count={}, max_move_bps={:.2})",
            self.window_ms(),
            self.jump_count(),
            self.max_move_bps()
        )
    }
}
