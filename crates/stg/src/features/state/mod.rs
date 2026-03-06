pub mod inventory;
pub mod cost;
pub mod latency;
pub mod rate_limit;

pub use inventory::{inv_skew, exposure_notional};
pub use cost::{cost_per_min_bps, cost_per_day_bps};
pub use latency::latency_p95_ms;
pub use rate_limit::rate_limit_risk_level;


#[derive(Debug, Clone)]
pub struct StateFeatures {
    pub inv_skew: f64,
    pub exposure_notional: f64,
    pub cost_per_min_bps: f64,
    pub cost_per_day_bps: f64,
    pub latency_p95_ms: f64,
    pub rate_limit_risk_level: f64,
}

impl StateFeatures {
    pub fn default_zero() -> Self {
        Self {
            inv_skew: 0.0,
            exposure_notional: 0.0,
            cost_per_min_bps: 0.0,
            cost_per_day_bps: 0.0,
            latency_p95_ms: 0.0,
            rate_limit_risk_level: 0.0,
        }
    }
}