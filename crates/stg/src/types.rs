use serde::{Deserialize, Serialize};

/// 市場方向
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Side {
    Buy,
    Sell,
}

/// Impact 查詢結果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactResult {
    pub ok: bool,
    pub vwap_price: f64,
    pub worst_price: f64,
    pub limit_price: f64,
    pub slip_bps: f64,
    pub used_level: usize,
}

impl Default for ImpactResult {
    fn default() -> Self {
        Self {
            ok: false,
            vwap_price: 0.0,
            worst_price: 0.0,
            limit_price: 0.0,
            slip_bps: 0.0,
            used_level: 0,
        }
    }
}

/// 流動性統計
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidityStats {
    pub bid1: f64,
    pub ask1: f64,
    pub mid: f64,
    pub spread_ticks: f64,
    pub spread_bps: f64,
    pub top_n_total_qty: f64,
    pub gap_ticks_bid: f64,
    pub gap_ticks_ask: f64,
}

impl Default for LiquidityStats {
    fn default() -> Self {
        Self {
            bid1: 0.0,
            ask1: 0.0,
            mid: 0.0,
            spread_ticks: 0.0,
            spread_bps: 0.0,
            top_n_total_qty: 0.0,
            gap_ticks_bid: 0.0,
            gap_ticks_ask: 0.0,
        }
    }
}

/// 訂單簿查詢介面
pub trait BookView {
    fn best(&self) -> LiquidityStats;
    fn impact(&self, side: Side, qty: f64, cushion_ticks: u32) -> ImpactResult;
    fn top_k(&self, side: Side, k: usize) -> (Vec<f64>, Vec<f64>);
}
