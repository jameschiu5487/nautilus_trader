pub mod types;
pub mod book;

// Re-export commonly used types
pub use book::{LocalL2Book, CostEstimatorConfig, MarketState};
pub use types::{Side, ImpactResult, LiquidityStats, BookView};

// Python bindings will be added later
#[cfg(feature = "python")]
pub mod python;
