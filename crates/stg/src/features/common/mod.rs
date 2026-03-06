pub mod spread;
pub mod gap;
pub mod microprice;
pub mod obi;
pub mod ofi;
pub mod volatility;
pub mod jump;
pub mod liquidity;


pub use spread::{spread_ticks, spread_bps};
pub use gap::{gap_ticks_bid, gap_ticks_ask};
pub use microprice::{microprice, mp_minus_mid_bps};
pub use obi::{obi_n, top_n_total_qty};


pub use ofi::{OfiCalculator, OrderEvent as OfiOrderEvent};
pub use volatility::{VolatilityCalculator, PriceTick as VolatilityPriceTick};
pub use jump::{JumpCalculator, PriceTick as JumpPriceTick};
pub use liquidity::{LiquidityCalculator, Trade as LiquidityTrade};


#[derive(Debug, Clone)]
pub struct CommonFeatures {
    pub spread_ticks: f64,
    pub spread_bps: f64,
    pub microprice: f64,
    pub mp_minus_mid_bps: f64,
    pub obi_n: f64,
    pub top_n_total_qty: f64,
    pub gap_ticks_bid: f64,
    pub gap_ticks_ask: f64,
    pub ofi_200ms: f64,
    pub ofi_1s: f64,
    pub cancel_ratio_200ms: f64,
    pub cancel_ratio_1s: f64,
    pub rvol_bps_200ms: f64,
    pub rvol_bps_1s: f64,
    pub jump_count_1s: u32,
    pub max_move_bps_1s: f64,
    pub liquidity_score: f64,
}

impl CommonFeatures {
    /// Create default zero features
    pub fn default_zero() -> Self {
        Self {
            spread_ticks: 0.0,
            spread_bps: 0.0,
            microprice: 0.0,
            mp_minus_mid_bps: 0.0,
            obi_n: 0.0,
            top_n_total_qty: 0.0,
            gap_ticks_bid: 0.0,
            gap_ticks_ask: 0.0,
            ofi_200ms: 0.0,
            ofi_1s: 0.0,
            cancel_ratio_200ms: 0.0,
            cancel_ratio_1s: 0.0,
            rvol_bps_200ms: 0.0,
            rvol_bps_1s: 0.0,
            jump_count_1s: 0,
            max_move_bps_1s: 0.0,
            liquidity_score: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_common_features_creation() {
        let features = CommonFeatures::default_zero();
        assert_eq!(features.spread_ticks, 0.0);
        assert_eq!(features.jump_count_1s, 0);
    }
}
