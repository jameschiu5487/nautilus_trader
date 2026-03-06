use std::collections::VecDeque;
const MS_TO_NS: u64 = 1_000_000;
const TRADING_DAYS_PER_YEAR: f64 = 365.0;
const TRADING_HOURS_PER_DAY: f64 = 24.0;
const SECONDS_PER_HOUR: f64 = 3600.0;
const ANNUALIZATION_FACTOR: f64 = TRADING_DAYS_PER_YEAR * TRADING_HOURS_PER_DAY * SECONDS_PER_HOUR;

#[derive(Debug, Clone, Copy)]
pub struct PriceTick {
    pub timestamp_ns: u64,
    pub price: f64,
}

pub struct VolatilityCalculator {
    window_ns: u64,
    prices: VecDeque<PriceTick>,
    log_returns: VecDeque<f64>,
    sum_sq_log_ret: f64,
    num_returns: usize,
}

impl VolatilityCalculator { 
    /// # Arguments
    /// * `window_ms` - Window size in milliseconds (e.g., 200, 300, 500, 1000)
    pub fn new(window_ms: u64) -> Self {
        Self {
            window_ns: window_ms * MS_TO_NS,
            prices: VecDeque::new(),
            log_returns: VecDeque::new(),
            sum_sq_log_ret: 0.0,
            num_returns: 0,
        }
    }
    
    pub fn window_ms(&self) -> u64 {
        self.window_ns / MS_TO_NS
    }
    
    pub fn update(&mut self, timestamp_ns: u64, price: f64) {
        if price <= 0.0 {
            return;
        }
        
        if let Some(last_tick) = self.prices.back() {
            let log_ret = (price / last_tick.price).ln();
            self.log_returns.push_back(log_ret);
            self.sum_sq_log_ret += log_ret * log_ret;
            self.num_returns += 1;
        }
        
        self.prices.push_back(PriceTick { timestamp_ns, price });
        self.clean_old_data(timestamp_ns);
    }
    
    pub fn rvol_bps(&self) -> f64 {
        if self.num_returns == 0 {
            return 0.0;
        }
        
        let var_per_tick = self.sum_sq_log_ret / self.num_returns as f64;
        let factor = (ANNUALIZATION_FACTOR / (self.window_ns as f64 / 1_000_000_000.0)) as f64;
        let annualized_vol = (var_per_tick * factor).sqrt();
        annualized_vol * 10_000.0
    }
    
    fn clean_old_data(&mut self, latest_timestamp_ns: u64) {
        let cutoff = latest_timestamp_ns.saturating_sub(self.window_ns);
        while let Some(tick) = self.prices.front() {
            if tick.timestamp_ns < cutoff {
                self.prices.pop_front();
                if let Some(log_ret) = self.log_returns.pop_front() {
                    self.sum_sq_log_ret -= log_ret * log_ret;
                    self.num_returns -= 1;
                }
            } else {
                break;
            }
        }
    }
}

impl Default for VolatilityCalculator {
    fn default() -> Self {
        Self::new(1000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_volatility_calculator_creation() {
        let calc = VolatilityCalculator::new(200);
        assert_eq!(calc.window_ms(), 200);
        assert_eq!(calc.rvol_bps(), 0.0);
    }
    
    #[test]
    fn test_volatility_single_price() {
        let mut calc = VolatilityCalculator::new(1000);
        calc.update(1000, 100.0);
        
        assert_eq!(calc.rvol_bps(), 0.0);
    }
    
    #[test]
    fn test_volatility_two_prices() {
        let mut calc = VolatilityCalculator::new(1000);
        calc.update(1000, 100.0);
        calc.update(2000, 101.0);
        calc.update(3000, 100.0);
        calc.update(4000, 101.0);
        assert!(calc.rvol_bps() > 0.0);
    }
    
    #[test]
    fn test_custom_window_500ms() {
        let mut calc = VolatilityCalculator::new(500);
        assert_eq!(calc.window_ms(), 500);
        
        calc.update(1000, 100.0);
        calc.update(2000, 101.0);
        assert!(calc.rvol_bps() > 0.0);
    }
}
