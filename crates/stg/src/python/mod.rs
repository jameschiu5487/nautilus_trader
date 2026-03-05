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

//! Python bindings for STG strategy components.

pub mod book;

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
    
    Ok(())
}
