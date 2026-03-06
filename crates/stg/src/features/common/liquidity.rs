use crate::types::Side;
use std::collections::VecDeque;

const MS_TO_NS: u64 = 1_000_000;

#[derive(Debug, Clone, Copy)]
pub struct Trade {
    pub timestamp_ns: u64,
    pub price: f64,
    pub size: f64,
    pub side: Side,
}

pub struct LiquidityCalculator {
    window_ns: u64,
    trades: VecDeque<Trade>,
    total_qty: f64,
    trade_count: usize,
}

impl LiquidityCalculator {
    /// # Arguments
    /// * `window_ms` - Window size in milliseconds (e.g., 500, 1000, 2000)
    pub fn new(window_ms: u64) -> Self {
        Self {
            window_ns: window_ms * MS_TO_NS,
            trades: VecDeque::new(),
            total_qty: 0.0,
            trade_count: 0,
        }
    }
    
    pub fn window_ms(&self) -> u64 {
        self.window_ns / MS_TO_NS
    }
    
    pub fn add_trade(&mut self, timestamp_ns: u64, price: f64, size: f64, side: Side) {
        let trade = Trade {
            timestamp_ns,
            price,
            size,
            side,
        };
        
        self.trades.push_back(trade);
        self.clean_old_trades(timestamp_ns);
        self.recompute_incremental_state();
    }
    
    pub fn avg_trade_qty(&self) -> f64 {
        if self.trade_count == 0 {
            0.0
        } else {
            self.total_qty / self.trade_count as f64
        }
    }
    
    pub fn liquidity_score(&self, top_n_total_qty: f64) -> f64 {
        let avg = self.avg_trade_qty();
        top_n_total_qty / (avg + 1e-6)
    }
    
    fn clean_old_trades(&mut self, latest_timestamp_ns: u64) {
        let cutoff = latest_timestamp_ns.saturating_sub(self.window_ns);
        
        while let Some(trade) = self.trades.front() {
            if trade.timestamp_ns < cutoff {
                self.trades.pop_front();
            } else {
                break;
            }
        }
    }
    
    fn recompute_incremental_state(&mut self) {
        self.total_qty = 0.0;
        self.trade_count = 0;
        
        for trade in &self.trades {
            self.total_qty += trade.size;
            self.trade_count += 1;
        }
    }
}

impl Default for LiquidityCalculator {
    /// Default to 1s window
    fn default() -> Self {
        Self::new(1000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_liquidity_calculator_creation() {
        let calc = LiquidityCalculator::new(1000);
        assert_eq!(calc.window_ms(), 1000);
        assert_eq!(calc.avg_trade_qty(), 0.0);
    }
    
    #[test]
    fn test_single_trade() {
        let mut calc = LiquidityCalculator::new(1000);
        calc.add_trade(1000, 100.0, 10.0, Side::Buy);
        
        assert!((calc.avg_trade_qty() - 10.0).abs() < 1e-6);
    }
    
    #[test]
    fn test_multiple_trades() {
        let mut calc = LiquidityCalculator::new(1000);
        calc.add_trade(1000, 100.0, 100.0, Side::Buy);
        calc.add_trade(2000, 100.05, 200.0, Side::Sell);
        
        assert!((calc.avg_trade_qty() - 150.0).abs() < 1e-6);
    }
    
    #[test]
    fn test_liquidity_score() {
        let mut calc = LiquidityCalculator::new(1000);
        calc.add_trade(1000, 100.0, 100.0, Side::Buy);
        calc.add_trade(2000, 100.05, 200.0, Side::Sell);
        
        let score = calc.liquidity_score(300.0);
        assert!((score - (300.0 / 150.0)).abs() < 1e-5);
    }
    
    #[test]
    fn test_custom_window_2s() {
        let mut calc = LiquidityCalculator::new(2000);
        assert_eq!(calc.window_ms(), 2000);
        
        calc.add_trade(1000, 100.0, 100.0, Side::Buy);
        assert!((calc.avg_trade_qty() - 100.0).abs() < 1e-6);
    }
}
