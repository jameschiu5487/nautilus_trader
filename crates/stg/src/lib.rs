pub mod types;
pub mod book;
pub mod features;

// Re-export commonly used types
pub use book::{LocalL2Book, CostEstimatorConfig, MarketState};
pub use types::{Side, ImpactResult, LiquidityStats, BookView};

// Re-export feature structs and calculators
pub use features::{CommonFeatures, QueueFeatures, StateFeatures};
pub use features::{OfiCalculator, VolatilityCalculator, LiquidityCalculator, JumpCalculator};

// Python bindings will be added later
#[cfg(feature = "python")]
pub mod python;
