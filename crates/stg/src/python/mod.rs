pub mod book;
pub mod features;

use pyo3::prelude::*;

/// Loaded as `stg` (standalone) or `nautilus_pyo3.stg` (integrated).
///
/// # Errors
///
/// Returns a `PyErr` if registering any module components fails.
#[pymodule]
pub fn stg(_: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Book Engine
    m.add_class::<book::PyLocalL2Book>()?;
    m.add_class::<book::PyMarketState>()?;
    m.add_class::<book::PyCostEstimatorConfig>()?;
    m.add_class::<book::PyLiquidityStats>()?;
    m.add_class::<book::PyImpactResult>()?;
    
    // Enums
    m.add_class::<book::PySide>()?;
    
    // Feature Calculators
    m.add_class::<features::PyOfiCalculator>()?;
    m.add_class::<features::PyVolatilityCalculator>()?;
    m.add_class::<features::PyLiquidityCalculator>()?;
    m.add_class::<features::PyJumpCalculator>()?;
    
    Ok(())
}
