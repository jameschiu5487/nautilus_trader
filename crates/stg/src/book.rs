use crate::types::{BookView, ImpactResult, LiquidityStats, Side};
#[derive(Debug, Clone)]
pub struct MarketState {
    /// Realized volatility (bps) in 1s window
    pub rvol_bps_1s: f64,
    /// Book Credibility [0, 1]
    pub bc: f64,
    /// Liquidity Crash Detector
    pub lcd: bool,
    /// Withdraw score after recent IOC [0, 1+]
    pub withdraw_score: f64,
    pub rate_limit_usage: f64,
    pub latency_p95_ms: f64,
}

impl Default for MarketState {
    fn default() -> Self {
        Self {
            rvol_bps_1s: 20.0,
            bc: 1.0,
            lcd: false,
            withdraw_score: 0.0,
            rate_limit_usage: 0.0,
            latency_p95_ms: 10.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CostEstimatorConfig {
    pub fee_taker_bps: f64,
    pub expected_exec_time_ms: f64,
    pub drift_rate_per_rvol: f64,
    pub bc_penalty_factor: f64,
    pub lcd_penalty_bps: f64,
    pub withdraw_penalty_factor: f64,
    pub rate_limit_penalty_factor: f64,
    pub latency_drift_rate_per_ms: f64,
    
    /// 流動性不足時的額外懲罰 (bps)
    /// 當訂單簿深度不足以完全執行時，在外推成本基礎上增加的固定懲罰
    pub insufficient_liquidity_penalty_bps: f64,
    
    /// 外推比例上限
    /// 當需要的數量超過可用流動性時，最多按此倍數外推 slippage
    pub extrapolation_ratio_limit: f64,
    
    /// 完全無流動性懲罰 (bps)
    /// 當訂單簿完全沒有對手方報價時的懲罰成本
    pub no_liquidity_penalty_bps: f64,
}

impl Default for CostEstimatorConfig {
    fn default() -> Self {
        Self {
            fee_taker_bps: 5.0,
            expected_exec_time_ms: 100.0,
            drift_rate_per_rvol: 0.5,
            bc_penalty_factor: 10.0,
            lcd_penalty_bps: 50.0,
            withdraw_penalty_factor: 5.0,
            rate_limit_penalty_factor: 10.0,
            latency_drift_rate_per_ms: 0.1,
            
            // 流動性不足懲罰參數
            insufficient_liquidity_penalty_bps: 50.0,  // 部分流動性不足的額外懲罰
            extrapolation_ratio_limit: 3.0,           // 外推比例上限（最多 3 倍）
            no_liquidity_penalty_bps: 200.0,          // 完全無流動性懲罰
        }
    }
}

pub struct LocalL2Book {
    top_k: usize,
    /// Bids: [0] = highest bid price (descending)
    bid_prices: Vec<f64>,
    bid_sizes: Vec<f64>,
    bid_cum_qty: Vec<f64>,    
    bid_cum_pq: Vec<f64>,
    /// Asks: [0] = lowest ask price (ascending)
    ask_prices: Vec<f64>,
    ask_sizes: Vec<f64>,
    ask_cum_qty: Vec<f64>,
    ask_cum_pq: Vec<f64>,
    tick_size: f64,
    cost_config: CostEstimatorConfig,
    
    /// topN: 計算 top_n_total_qty 時使用的檔位數量
    /// 規格 §12: topK (50/100/200), topN (10/20)
    top_n: usize,
}

impl LocalL2Book {
    pub fn new(top_k: usize, tick_size: f64) -> Self {
        Self::with_cost_config(top_k, tick_size, CostEstimatorConfig::default())
    }
    
    pub fn with_cost_config(
        top_k: usize,
        tick_size: f64,
        cost_config: CostEstimatorConfig,
    ) -> Self {
        Self::with_full_config(top_k, 10, tick_size, cost_config)
    }
    
    /// 完整配置建構函數
    /// 
    /// # Arguments
    /// * `top_k` - 維護的最大檔位數量 (§12: 50/100/200)
    /// * `top_n` - 計算 top_n_total_qty 的檔位數量 (§12: 10/20)
    /// * `tick_size` - 最小價格變動單位
    /// * `cost_config` - 成本估算配置
    pub fn with_full_config(
        top_k: usize,
        top_n: usize,
        tick_size: f64,
        cost_config: CostEstimatorConfig,
    ) -> Self {
        Self {
            top_k,
            bid_prices: Vec::with_capacity(top_k),
            bid_sizes: Vec::with_capacity(top_k),
            bid_cum_qty: Vec::with_capacity(top_k),
            bid_cum_pq: Vec::with_capacity(top_k),
            ask_prices: Vec::with_capacity(top_k),
            ask_sizes: Vec::with_capacity(top_k),
            ask_cum_qty: Vec::with_capacity(top_k),
            ask_cum_pq: Vec::with_capacity(top_k),
            tick_size,
            cost_config,
            top_n,
        }
    }
    
    pub fn update(&mut self, side: Side, price: f64, size: f64) {
        match side {
            Side::Buy => Self::update_side(
                &mut self.bid_prices,
                &mut self.bid_sizes,
                &mut self.bid_cum_qty,
                &mut self.bid_cum_pq,
                price,
                size,
                true, // descending
                self.top_k,
                self.tick_size,
            ),
            Side::Sell => Self::update_side(
                &mut self.ask_prices,
                &mut self.ask_sizes,
                &mut self.ask_cum_qty,
                &mut self.ask_cum_pq,
                price,
                size,
                false, // ascending
                self.top_k,
                self.tick_size,
            ),
        }
    }
    
    fn update_side(
        prices: &mut Vec<f64>,
        sizes: &mut Vec<f64>,
        cum_qty: &mut Vec<f64>,
        cum_pq: &mut Vec<f64>,
        price: f64,
        size: f64,
        descending: bool,
        top_k: usize,
        _tick_size: f64,
    ) {
        // Find position using binary search
        let pos = if descending {
            prices.binary_search_by(|p| price.partial_cmp(p).unwrap()).unwrap_or_else(|e| e)
        } else {
            prices.binary_search_by(|p| p.partial_cmp(&price).unwrap()).unwrap_or_else(|e| e)
        };
        
        let update_from_pos = if pos < prices.len() && (prices[pos] - price).abs() < 1e-10 {
            // Update existing level
            if size == 0.0 {
                // Remove level - need to update from this position
                prices.remove(pos);
                sizes.remove(pos);
                cum_qty.remove(pos);
                cum_pq.remove(pos);
                pos
            } else {
                // Modify existing level - need to update from this position
                sizes[pos] = size;
                pos
            }
        } else if size > 0.0 {
            // Insert new level - need to update from this position
            prices.insert(pos, price);
            sizes.insert(pos, size);
            // Insert placeholder values (will be updated below)
            cum_qty.insert(pos, 0.0);
            cum_pq.insert(pos, 0.0);
            pos
        } else {
            // No change (tried to delete non-existent level)
            return;
        };
        
        // Trim to topK
        if prices.len() > top_k {
            prices.truncate(top_k);
            sizes.truncate(top_k);
            cum_qty.truncate(top_k);
            cum_pq.truncate(top_k);
        }
        
        // Incremental update: only update from affected position onwards
        Self::update_prefix_sums_from(prices, sizes, cum_qty, cum_pq, update_from_pos);
    }
    
    fn update_prefix_sums_from(
        prices: &[f64],
        sizes: &[f64],
        cum_qty: &mut [f64],
        cum_pq: &mut [f64],
        start_pos: usize,
    ) {
        if start_pos >= prices.len() {
            return;
        }
        
        // Get base values from previous position
        let (base_qty, base_pq) = if start_pos > 0 {
            (cum_qty[start_pos - 1], cum_pq[start_pos - 1])
        } else {
            (0.0, 0.0)
        };
        
        let mut running_qty = base_qty;
        let mut running_pq = base_pq;
        
        // Update from start_pos onwards
        for i in start_pos..prices.len() {
            running_qty += sizes[i];
            running_pq += prices[i] * sizes[i];
            cum_qty[i] = running_qty;
            cum_pq[i] = running_pq;
        }
    }
    
    #[allow(dead_code)]
    fn recompute_prefix_sums(
        prices: &[f64],
        sizes: &[f64],
        cum_qty: &mut Vec<f64>,
        cum_pq: &mut Vec<f64>,
    ) {
        cum_qty.clear();
        cum_pq.clear();
        
        let mut running_qty = 0.0;
        let mut running_pq = 0.0;
        
        for (price, size) in prices.iter().zip(sizes.iter()) {
            running_qty += size;
            running_pq += price * size;
            cum_qty.push(running_qty);
            cum_pq.push(running_pq);
        }
    }

    fn compute_impact_internal(
        &self,
        prices: &[f64],
        _sizes: &[f64],  // Unused but kept for potential future use
        cum_qty: &[f64],
        cum_pq: &[f64],
        qty: f64,
        cushion_ticks: u32,
        side: Side,
    ) -> ImpactResult {
        if prices.is_empty() || qty <= 0.0 {
            return ImpactResult::default();
        }
        
        // Find level where cum_qty >= qty (binary search)
        let used_level = match cum_qty.binary_search_by(|cq| {
            if *cq < qty {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Greater
            }
        }) {
            Ok(i) | Err(i) => i,
        };
        
        if used_level >= cum_qty.len() {
            // Not enough liquidity
            return ImpactResult {
                ok: false,
                ..Default::default()
            };
        }
        
        let vwap = if used_level == 0 {
            prices[0]
        } else {
            let prev_cum_pq = if used_level > 0 { cum_pq[used_level - 1] } else { 0.0 };
            let prev_cum_qty = if used_level > 0 { cum_qty[used_level - 1] } else { 0.0 };
            let remaining = qty - prev_cum_qty;
            (prev_cum_pq + remaining * prices[used_level]) / qty
        };
        
        let worst_price = prices[used_level];
        
        let cushion = cushion_ticks as f64 * self.tick_size;
        let limit_price = match side {
            Side::Sell => worst_price - cushion,
            Side::Buy => worst_price + cushion,
        };
        
        let mid = self.best().mid;
        let slip_bps = if mid > 0.0 {
            let adverse_dir = match side {
                Side::Sell => -1.0,
                Side::Buy => 1.0,
            };
            adverse_dir * (vwap - mid) / mid * 10000.0
        } else {
            0.0
        };
        
        ImpactResult {
            ok: true,
            vwap_price: vwap,
            worst_price,
            limit_price,
            slip_bps,
            used_level,
        }
    }

    pub fn bailout_cost_bps_est(
        &self,
        qty: f64,
        side: Side,
        market_state: &MarketState,
    ) -> f64 {
        if qty <= 0.0 {
            return 0.0;
        }
        
        let config = &self.cost_config;
        
        let fee_bps = config.fee_taker_bps;
        
        let impact = self.impact(side, qty, 0);
        let slip_bps = if impact.ok {
            impact.slip_bps
        } else {
            self.estimate_insufficient_liquidity_cost(side, qty)
        };
        
        let adverse_drift_bps = config.drift_rate_per_rvol 
            * market_state.rvol_bps_1s 
            * (config.expected_exec_time_ms / 1000.0);
        
        let latency_penalty_bps = config.latency_drift_rate_per_ms 
            * market_state.latency_p95_ms;
        
        let bc_penalty_bps = config.bc_penalty_factor * (1.0 - market_state.bc);
        
        let lcd_penalty_bps = if market_state.lcd {
            config.lcd_penalty_bps
        } else {
            0.0
        };
        
        let withdraw_penalty_bps = config.withdraw_penalty_factor 
            * market_state.withdraw_score;
        
        let rate_limit_penalty_bps = config.rate_limit_penalty_factor 
            * market_state.rate_limit_usage;
        
        fee_bps
            + slip_bps
            + adverse_drift_bps
            + latency_penalty_bps
            + bc_penalty_bps
            + lcd_penalty_bps
            + withdraw_penalty_bps
            + rate_limit_penalty_bps
    }
    
    fn estimate_insufficient_liquidity_cost(&self, side: Side, qty: f64) -> f64 {
        let available_qty = match side {
            Side::Sell => self.bid_cum_qty.last().copied().unwrap_or(0.0),
            Side::Buy => self.ask_cum_qty.last().copied().unwrap_or(0.0),
        };
        
        if available_qty > 0.0 {
            let partial_impact = self.impact(side, available_qty, 0);
            let ratio = (qty / available_qty).min(self.cost_config.extrapolation_ratio_limit);
            partial_impact.slip_bps * ratio + self.cost_config.insufficient_liquidity_penalty_bps
        } else {
            self.cost_config.no_liquidity_penalty_bps
        }
    }
    
    pub fn cost_config(&self) -> &CostEstimatorConfig {
        &self.cost_config
    }
    
    pub fn set_cost_config(&mut self, config: CostEstimatorConfig) {
        self.cost_config = config;
    }
    
    pub fn microprice(&self) -> Option<f64> {
        let bid1 = self.bid_prices.first().copied()?;
        let ask1 = self.ask_prices.first().copied()?;
        let bid1_qty = self.bid_sizes.first().copied()?;
        let ask1_qty = self.ask_sizes.first().copied()?;
        
        if bid1 <= 0.0 || ask1 <= 0.0 || bid1_qty <= 0.0 || ask1_qty <= 0.0 {
            return None;
        }
        
        let total_qty = bid1_qty + ask1_qty;
        if total_qty <= 0.0 {
            return None;
        }
        
        let mp = (bid1 * ask1_qty + ask1 * bid1_qty) / total_qty;
        
        Some(mp)
    }
    
    pub fn microprice_deviation_bps(&self) -> Option<f64> {
        let mp = self.microprice()?;
        let stats = self.best();
        
        if stats.mid <= 0.0 {
            return None;
        }
        
        Some((mp - stats.mid) / stats.mid * 10000.0)
    }
}

impl BookView for LocalL2Book {
    fn best(&self) -> LiquidityStats {
        let bid1 = self.bid_prices.first().copied().unwrap_or(0.0);
        let ask1 = self.ask_prices.first().copied().unwrap_or(0.0);
        let mid = if bid1 > 0.0 && ask1 > 0.0 {
            (bid1 + ask1) / 2.0
        } else {
            0.0
        };

        let spread_ticks = if bid1 > 0.0 && ask1 > 0.0 {
            (ask1 - bid1) / self.tick_size
        } else {
            0.0
        };

        let spread_bps = if mid > 0.0 {
            ((ask1 - bid1) / mid) * 10000.0
        } else {
            0.0
        };

        // Calculate top N total qty (use min of top_n and available levels)
        let n = self.top_n.min(self.bid_sizes.len()).min(self.ask_sizes.len());
        let top_n_total_qty: f64 = self.bid_sizes.iter().take(n).sum::<f64>()
            + self.ask_sizes.iter().take(n).sum::<f64>();

        // Calculate gaps
        let gap_ticks_bid = if self.bid_prices.len() >= 2 {
            (self.bid_prices[0] - self.bid_prices[1]) / self.tick_size
        } else {
            0.0
        };

        let gap_ticks_ask = if self.ask_prices.len() >= 2 {
            (self.ask_prices[1] - self.ask_prices[0]) / self.tick_size
        } else {
            0.0
        };

        LiquidityStats {
            bid1,
            ask1,
            mid,
            spread_ticks,
            spread_bps,
            top_n_total_qty,
            gap_ticks_bid,
            gap_ticks_ask,
        }
    }
    
    fn impact(&self, side: Side, qty: f64, cushion_ticks: u32) -> ImpactResult {
        match side {
            Side::Sell => self.compute_impact_internal(
                &self.bid_prices,
                &self.bid_sizes,
                &self.bid_cum_qty,
                &self.bid_cum_pq,
                qty,
                cushion_ticks,
                side,
            ),
            Side::Buy => self.compute_impact_internal(
                &self.ask_prices,
                &self.ask_sizes,
                &self.ask_cum_qty,
                &self.ask_cum_pq,
                qty,
                cushion_ticks,
                side,
            ),
        }
    }
    
    fn top_k(&self, side: Side, k: usize) -> (Vec<f64>, Vec<f64>) {
        match side {
            Side::Buy => (
                self.bid_prices.iter().take(k).copied().collect(),
                self.bid_sizes.iter().take(k).copied().collect(),
            ),
            Side::Sell => (
                self.ask_prices.iter().take(k).copied().collect(),
                self.ask_sizes.iter().take(k).copied().collect(),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_empty_book() {
        let book = LocalL2Book::new(100, 0.01);
        let stats = book.best();
        assert_eq!(stats.mid, 0.0);
    }
    
    #[test]
    fn test_single_level_update() {
        let mut book = LocalL2Book::new(100, 0.01);
        book.update(Side::Buy, 100.0, 10.0);
        book.update(Side::Sell, 101.0, 10.0);
        
        let stats = book.best();
        assert_eq!(stats.bid1, 100.0);
        assert_eq!(stats.ask1, 101.0);
        assert_eq!(stats.mid, 100.5);
    }
    
    #[test]
    fn test_impact_single_level() {
        let mut book = LocalL2Book::new(100, 0.01);
        book.update(Side::Buy, 100.0, 10.0);
        book.update(Side::Sell, 101.0, 10.0);
        
        let impact = book.impact(Side::Sell, 5.0, 2);
        assert!(impact.ok);
        assert_eq!(impact.vwap_price, 100.0);
        assert_eq!(impact.used_level, 0);
    }
    
    #[test]
    fn test_slippage_calculation() {
        let mut book = LocalL2Book::new(100, 0.01);
        book.update(Side::Buy, 100.0, 10.0);   // bid1 = 100
        book.update(Side::Sell, 101.0, 10.0);  // ask1 = 101, mid = 100.5
        let impact_sell = book.impact(Side::Sell, 5.0, 0);
        assert!(impact_sell.ok);
        assert!((impact_sell.slip_bps - 49.75).abs() < 0.1, 
            "Expected slip_bps ~49.75, got {:.2}", impact_sell.slip_bps);
    
        let impact_buy = book.impact(Side::Buy, 5.0, 0);
        assert!(impact_buy.ok);
        assert!((impact_buy.slip_bps - 49.75).abs() < 0.1,
            "Expected slip_bps ~49.75, got {:.2}", impact_buy.slip_bps);
        
        println!("Sell slip_bps: {:.2} (should be positive = cost)", impact_sell.slip_bps);
        println!("Buy slip_bps: {:.2} (should be positive = cost)", impact_buy.slip_bps);
    }
    
    #[test]
    fn test_impact_multi_level() {
        let mut book = LocalL2Book::new(100, 0.01);
        book.update(Side::Buy, 100.0, 10.0);
        book.update(Side::Buy, 99.9, 20.0);
        book.update(Side::Buy, 99.8, 30.0);
        
        let impact = book.impact(Side::Sell, 25.0, 2);
        assert!(impact.ok);
        assert!(impact.used_level >= 1);
        assert!(impact.vwap_price < 100.0);
        assert!(impact.vwap_price > 99.9);
    }
    
    #[test]
    fn test_bailout_cost_basic() {
        let mut book = LocalL2Book::new(100, 0.01);
        book.update(Side::Buy, 100.0, 10.0);
        book.update(Side::Sell, 101.0, 10.0);
        let default_state = MarketState::default();
        let cost_bps = book.bailout_cost_bps_est(5.0, Side::Sell, &default_state);
        assert!(cost_bps > 0.0);
        assert!(cost_bps >= 5.0);
        println!("Bailout cost for 5 units: {:.2} bps", cost_bps);
    }
    
    #[test]
    fn test_bailout_cost_with_market_state() {
        let mut book = LocalL2Book::new(100, 0.01);
        book.update(Side::Buy, 100.0, 10.0);
        book.update(Side::Sell, 101.0, 10.0);
        
        // Normal market state
        let normal_state = MarketState {
            rvol_bps_1s: 20.0,
            bc: 1.0,
            lcd: false,
            withdraw_score: 0.0,
            rate_limit_usage: 0.0,
            latency_p95_ms: 10.0,
        };
        
        let cost_normal = book.bailout_cost_bps_est(5.0, Side::Sell, &normal_state);
        
        // Stressed market state
        let stressed_state = MarketState {
            rvol_bps_1s: 100.0,  
            bc: 0.3,              
            lcd: true,            
            withdraw_score: 0.8,  
            rate_limit_usage: 0.9, 
            latency_p95_ms: 50.0, 
        };
        
        let cost_stressed = book.bailout_cost_bps_est(5.0, Side::Sell, &stressed_state);
        
        println!("Normal market cost: {:.2} bps", cost_normal);
        println!("Stressed market cost: {:.2} bps", cost_stressed);
        assert!(cost_stressed > cost_normal * 2.0,
            "Stressed cost ({:.2}) should be >> normal cost ({:.2})",
            cost_stressed, cost_normal);
    }
    
    #[test]
    fn test_bailout_cost_multi_level() {
        let mut book = LocalL2Book::new(100, 0.01);
        
        // Setup multi-level book
        book.update(Side::Buy, 100.0, 10.0);
        book.update(Side::Buy, 99.9, 20.0);
        book.update(Side::Buy, 99.8, 30.0);
        book.update(Side::Sell, 101.0, 10.0);  // Add ask for mid calculation
        
        let state = MarketState::default();
        
        // Small quantity - should be cheaper
        let cost_small = book.bailout_cost_bps_est(5.0, Side::Sell, &state);
        
        // Large quantity - should be more expensive (more slippage)
        let cost_large = book.bailout_cost_bps_est(25.0, Side::Sell, &state);
        
        println!("Small qty (5) cost: {:.2} bps", cost_small);
        println!("Large qty (25) cost: {:.2} bps", cost_large);
        let impact_small = book.impact(Side::Sell, 5.0, 0);
        let impact_large = book.impact(Side::Sell, 25.0, 0);
        println!("Small qty slip: {:.2} bps", impact_small.slip_bps);
        println!("Large qty slip: {:.2} bps", impact_large.slip_bps);
        
        assert!(cost_large > cost_small, 
            "Expected large qty cost ({:.2}) > small qty cost ({:.2})", 
            cost_large, cost_small);
    }
    
    #[test]
    fn test_bailout_cost_insufficient_liquidity() {
        let mut book = LocalL2Book::new(100, 0.01);
        book.update(Side::Buy, 100.0, 10.0);
        book.update(Side::Sell, 101.0, 10.0);
        let state = MarketState::default();
        let cost_bps = book.bailout_cost_bps_est(50.0, Side::Sell, &state);
        assert!(cost_bps > 50.0);
        
        println!("Insufficient liquidity bailout cost: {:.2} bps", cost_bps);
    }
    
    #[test]
    fn test_bailout_cost_zero_qty() {
        let book = LocalL2Book::new(100, 0.01);
        let state = MarketState::default();
        let cost_bps = book.bailout_cost_bps_est(0.0, Side::Sell, &state);
        assert_eq!(cost_bps, 0.0);
    }
    
    #[test]
    fn test_bailout_cost_custom_config() {
        let aggressive_config = CostEstimatorConfig {
            fee_taker_bps: 10.0,
            expected_exec_time_ms: 500.0,
            drift_rate_per_rvol: 1.0,
            bc_penalty_factor: 20.0,
            lcd_penalty_bps: 100.0,
            withdraw_penalty_factor: 10.0,
            rate_limit_penalty_factor: 20.0,
            latency_drift_rate_per_ms: 0.2,
            insufficient_liquidity_penalty_bps: 50.0,
            extrapolation_ratio_limit: 3.0,
            no_liquidity_penalty_bps: 200.0,
        };
        
        let mut book = LocalL2Book::with_cost_config(100, 0.01, aggressive_config);
        book.update(Side::Buy, 100.0, 10.0);
        book.update(Side::Sell, 101.0, 10.0);
        
        let state = MarketState::default();
        let cost_bps = book.bailout_cost_bps_est(5.0, Side::Sell, &state);
        
        // Should be significantly higher with aggressive config
        assert!(cost_bps > 30.0);
        
        println!("Aggressive config bailout cost: {:.2} bps", cost_bps);
    }
    
    #[test]
    fn test_bailout_cost_components() {
        let mut book = LocalL2Book::new(100, 0.01);
        book.update(Side::Buy, 100.0, 10.0);
        book.update(Side::Sell, 101.0, 10.0);
        
        // Use known market state for testing
        let test_state = MarketState {
            rvol_bps_1s: 20.0,
            bc: 1.0,
            lcd: false,
            withdraw_score: 0.0,
            rate_limit_usage: 0.0,
            latency_p95_ms: 10.0,
        };
        
        let total_cost = book.bailout_cost_bps_est(5.0, Side::Sell, &test_state);
        
        let config = book.cost_config();
        let fee = config.fee_taker_bps; // 5.0
        let drift = config.drift_rate_per_rvol * 20.0 * 0.1; // 0.5 * 20 * 0.1 = 1.0
        let latency = config.latency_drift_rate_per_ms * 10.0; // 0.1 * 10 = 1.0
        
        let fixed_components = fee + drift + latency; // ~7.0 bps
        
        // Total should be at least the fixed components (+ slippage)
        assert!(total_cost >= fixed_components);
        
        println!("Total cost: {:.2} bps", total_cost);
        println!("Fixed components: {:.2} bps", fixed_components);
        println!("Implied slippage: {:.2} bps", total_cost - fixed_components);
    }
    
    #[test]
    fn test_level_deletion() {
        let mut book = LocalL2Book::new(100, 0.01);
        
        // Add levels
        book.update(Side::Buy, 100.0, 10.0);
        book.update(Side::Buy, 99.9, 20.0);
        book.update(Side::Buy, 99.8, 30.0);
        
        let (prices, sizes) = book.top_k(Side::Buy, 10);
        assert_eq!(prices.len(), 3);
        assert_eq!(sizes.len(), 3);
        
        // Delete middle level (size = 0)
        book.update(Side::Buy, 99.9, 0.0);
        
        let (prices, sizes) = book.top_k(Side::Buy, 10);
        assert_eq!(prices.len(), 2);
        assert_eq!(sizes.len(), 2);
        assert_eq!(prices[0], 100.0);
        assert_eq!(prices[1], 99.8);
        
        println!("After deletion: prices={:?}, sizes={:?}", prices, sizes);
    }
    
    #[test]
    fn test_top_k_limit() {
        let mut book = LocalL2Book::new(5, 0.01); // topK = 5
        
        // Add more than topK levels
        for i in 0..10 {
            book.update(Side::Buy, 100.0 - i as f64 * 0.1, 10.0);
        }
        
        let (prices, _sizes) = book.top_k(Side::Buy, 20);
        
        // Should only have 5 levels (topK limit)
        assert_eq!(prices.len(), 5);
        assert_eq!(prices[0], 100.0);
        assert_eq!(prices[4], 99.6);
        
        println!("TopK limit: kept {} levels out of 10", prices.len());
    }
    
    #[test]
    fn test_prefix_sums_correctness() {
        let mut book = LocalL2Book::new(100, 0.01);
        
        // Add known levels
        book.update(Side::Buy, 100.0, 10.0);
        book.update(Side::Buy, 99.9, 20.0);
        book.update(Side::Buy, 99.8, 30.0);
        
        // Manually calculate expected prefix sums
        // cum_qty: [10, 30, 60]
        // cum_pq: [1000, 1000+1998, 1000+1998+2994] = [1000, 2998, 5992]
        
        // Test via impact (which uses prefix sums)
        let impact_10 = book.impact(Side::Sell, 10.0, 0);
        assert!(impact_10.ok);
        assert_eq!(impact_10.vwap_price, 100.0);
        assert_eq!(impact_10.used_level, 0);
        
        let impact_25 = book.impact(Side::Sell, 25.0, 0);
        assert!(impact_25.ok);
        assert_eq!(impact_25.used_level, 1);
        // VWAP = (10*100 + 15*99.9) / 25 = (1000 + 1498.5) / 25 = 99.94
        assert!((impact_25.vwap_price - 99.94).abs() < 0.01);
        
        let impact_60 = book.impact(Side::Sell, 60.0, 0);
        assert!(impact_60.ok);
        assert_eq!(impact_60.used_level, 2);
        // VWAP = (10*100 + 20*99.9 + 30*99.8) / 60 = 5992 / 60 = 99.8667
        assert!((impact_60.vwap_price - 99.8667).abs() < 0.01);
        
        println!("Prefix sums test passed: cum_qty and cum_pq are correct");
    }
    
    #[test]
    fn test_qty_exactly_at_level_boundary() {
        let mut book = LocalL2Book::new(100, 0.01);
        
        book.update(Side::Buy, 100.0, 10.0);
        book.update(Side::Buy, 99.9, 20.0);
        book.update(Side::Buy, 99.8, 30.0);
        
        // Test qty exactly equal to first level
        let impact_10 = book.impact(Side::Sell, 10.0, 0);
        assert!(impact_10.ok);
        assert_eq!(impact_10.vwap_price, 100.0);
        assert_eq!(impact_10.used_level, 0);
        
        // Test qty exactly equal to first two levels combined
        let impact_30 = book.impact(Side::Sell, 30.0, 0);
        assert!(impact_30.ok);
        assert_eq!(impact_30.used_level, 1);
        // VWAP = (10*100 + 20*99.9) / 30 = 99.933...
        assert!((impact_30.vwap_price - 99.9333).abs() < 0.001);
        
        println!("Boundary test: qty=10 uses level 0, qty=30 uses level 1");
    }
    
    #[test]
    fn test_qty_exceeds_all_liquidity() {
        let mut book = LocalL2Book::new(100, 0.01);
        
        book.update(Side::Buy, 100.0, 10.0);
        book.update(Side::Buy, 99.9, 20.0);
        // Total available: 30 units
        
        // Request 100 units (way more than available)
        let impact = book.impact(Side::Sell, 100.0, 0);
        
        // Should fail (not enough liquidity)
        assert!(!impact.ok);
        
        println!("Insufficient liquidity: requested 100, available 30 -> fail");
    }
    
    #[test]
    fn test_one_sided_book_bid_only() {
        let mut book = LocalL2Book::new(100, 0.01);
        
        // Only add bids, no asks
        book.update(Side::Buy, 100.0, 10.0);
        book.update(Side::Buy, 99.9, 20.0);
        
        let stats = book.best();
        
        // bid1 should be valid
        assert_eq!(stats.bid1, 100.0);
        
        // ask1 should be 0 (no asks)
        assert_eq!(stats.ask1, 0.0);
        
        // mid should be 0 (can't calculate without both sides)
        assert_eq!(stats.mid, 0.0);
        
        // Can impact on bid side
        let impact_sell = book.impact(Side::Sell, 5.0, 0);
        assert!(impact_sell.ok);
        
        // Cannot impact on ask side (no liquidity)
        let impact_buy = book.impact(Side::Buy, 5.0, 0);
        assert!(!impact_buy.ok);
        
        println!("One-sided book (bid only): Sell ok, Buy fails");
    }
    
    #[test]
    fn test_one_sided_book_ask_only() {
        let mut book = LocalL2Book::new(100, 0.01);
        
        // Only add asks, no bids
        book.update(Side::Sell, 101.0, 10.0);
        book.update(Side::Sell, 101.1, 20.0);
        
        let stats = book.best();
        
        // ask1 should be valid
        assert_eq!(stats.ask1, 101.0);
        
        // bid1 should be 0 (no bids)
        assert_eq!(stats.bid1, 0.0);
        
        // mid should be 0 (can't calculate without both sides)
        assert_eq!(stats.mid, 0.0);
        
        // Can impact on ask side
        let impact_buy = book.impact(Side::Buy, 5.0, 0);
        assert!(impact_buy.ok);
        
        // Cannot impact on bid side (no liquidity)
        let impact_sell = book.impact(Side::Sell, 5.0, 0);
        assert!(!impact_sell.ok);
        
        println!("One-sided book (ask only): Buy ok, Sell fails");
    }
    
    #[test]
    fn test_empty_book_impact() {
        let book = LocalL2Book::new(100, 0.01);
        
        // Empty book - no liquidity on either side
        let impact_sell = book.impact(Side::Sell, 10.0, 0);
        assert!(!impact_sell.ok);
        
        let impact_buy = book.impact(Side::Buy, 10.0, 0);
        assert!(!impact_buy.ok);
        
        println!("Empty book: both sides fail");
    }
    
    #[test]
    fn test_incremental_prefix_sums() {
        let mut book = LocalL2Book::new(100, 0.01);
        
        // Build book incrementally
        book.update(Side::Buy, 100.0, 10.0);
        book.update(Side::Buy, 99.9, 20.0);
        book.update(Side::Buy, 99.8, 30.0);
        
        // Verify prefix sums after incremental updates
        let impact_60 = book.impact(Side::Sell, 60.0, 0);
        assert!(impact_60.ok);
        assert_eq!(impact_60.used_level, 2);
        assert!((impact_60.vwap_price - 99.8667).abs() < 0.01);
        
        // Modify middle level (should trigger incremental update)
        book.update(Side::Buy, 99.9, 25.0); // Change from 20 to 25
        
        // New total: 10 + 25 + 30 = 65
        // New cum_pq: 1000 + 2497.5 + 2994 = 6491.5
        // VWAP(65) = 6491.5 / 65 = 99.8692...
        let impact_65 = book.impact(Side::Sell, 65.0, 0);
        assert!(impact_65.ok);
        assert_eq!(impact_65.used_level, 2);
        assert!((impact_65.vwap_price - 99.8692).abs() < 0.01);
        
        // Delete first level (should trigger incremental update from pos 0)
        book.update(Side::Buy, 100.0, 0.0);
        
        // Now only 2 levels: 99.9 (25) and 99.8 (30)
        // Total: 55
        let impact_55 = book.impact(Side::Sell, 55.0, 0);
        assert!(impact_55.ok);
        assert_eq!(impact_55.used_level, 1);
        // VWAP = (25*99.9 + 30*99.8) / 55 = 99.8409...
        assert!((impact_55.vwap_price - 99.8409).abs() < 0.01);
        
        println!("Incremental prefix sums: all updates verified correct");
    }
    
    #[test]
    fn test_microprice_balanced_book() {
        let mut book = LocalL2Book::new(100, 0.01);
        
        // Balanced book: equal quantities on both sides
        book.update(Side::Buy, 100.0, 10.0);
        book.update(Side::Sell, 100.1, 10.0);
        
        let mp = book.microprice().unwrap();
        let mid = book.best().mid;
        
        // When quantities are equal, microprice = mid-price
        // mp = (100.0 * 10 + 100.1 * 10) / 20 = 100.05
        assert!((mp - mid).abs() < 1e-6);
        assert!((mp - 100.05).abs() < 1e-6);
        
        let dev_bps = book.microprice_deviation_bps().unwrap();
        assert!(dev_bps.abs() < 0.1); // Should be near zero
        
        println!("Balanced book: microprice = mid = {}", mp);
    }
    
    #[test]
    fn test_microprice_bid_heavy() {
        let mut book = LocalL2Book::new(100, 0.01);
        
        // Bid-heavy book: more quantity on bid side
        book.update(Side::Buy, 100.0, 20.0);  // Large bid
        book.update(Side::Sell, 100.1, 5.0);  // Small ask
        
        let mp = book.microprice().unwrap();
        let mid = book.best().mid;
        
        // mp = (100.0 * 5 + 100.1 * 20) / 25 = 100.08
        // mid = (100.0 + 100.1) / 2 = 100.05
        // mp > mid (買方壓力大)
        assert!((mp - 100.08).abs() < 1e-6);
        assert!(mp > mid);
        
        let dev_bps = book.microprice_deviation_bps().unwrap();
        // dev = (100.08 - 100.05) / 100.05 * 10000 = 3.0 bps
        assert!((dev_bps - 3.0).abs() < 0.1);
        assert!(dev_bps > 0.0); // Positive = buy pressure
        
        println!("Bid-heavy: mp={}, mid={}, dev={:.2} bps", mp, mid, dev_bps);
    }
    
    #[test]
    fn test_microprice_ask_heavy() {
        let mut book = LocalL2Book::new(100, 0.01);
        
        // Ask-heavy book: more quantity on ask side
        book.update(Side::Buy, 100.0, 5.0);   // Small bid
        book.update(Side::Sell, 100.1, 20.0); // Large ask
        
        let mp = book.microprice().unwrap();
        let mid = book.best().mid;
        
        // mp = (100.0 * 20 + 100.1 * 5) / 25 = 100.02
        // mid = (100.0 + 100.1) / 2 = 100.05
        // mp < mid (賣方壓力大)
        assert!((mp - 100.02).abs() < 1e-6);
        assert!(mp < mid);
        
        let dev_bps = book.microprice_deviation_bps().unwrap();
        // dev = (100.02 - 100.05) / 100.05 * 10000 = -3.0 bps
        assert!((dev_bps + 3.0).abs() < 0.1);
        assert!(dev_bps < 0.0); // Negative = sell pressure
        
        println!("Ask-heavy: mp={}, mid={}, dev={:.2} bps", mp, mid, dev_bps);
    }
    
    #[test]
    fn test_microprice_one_sided() {
        let mut book = LocalL2Book::new(100, 0.01);
        
        // Only bid side
        book.update(Side::Buy, 100.0, 10.0);
        assert!(book.microprice().is_none());
        
        // Only ask side
        let mut book2 = LocalL2Book::new(100, 0.01);
        book2.update(Side::Sell, 100.1, 10.0);
        assert!(book2.microprice().is_none());
        
        println!("One-sided books: microprice = None (as expected)");
    }
    
    #[test]
    fn test_microprice_empty_book() {
        let book = LocalL2Book::new(100, 0.01);
        
        assert!(book.microprice().is_none());
        assert!(book.microprice_deviation_bps().is_none());
        
        println!("Empty book: microprice = None (as expected)");
    }
    
    #[test]
    fn test_top_n_configuration() {
        // Test with top_n = 5 (smaller than default 10)
        let mut book = LocalL2Book::with_full_config(
            100,
            5,  // top_n = 5
            0.01,
            CostEstimatorConfig::default()
        );
        
        // Add 10 levels on each side
        for i in 0..10 {
            book.update(Side::Buy, 100.0 - i as f64 * 0.1, 10.0);
            book.update(Side::Sell, 100.1 + i as f64 * 0.1, 10.0);
        }
        
        let stats = book.best();
        
        // top_n_total_qty should only include top 5 levels on each side
        // 5 levels * 10 qty * 2 sides = 100
        assert!((stats.top_n_total_qty - 100.0).abs() < 0.01);
        
        println!("top_n=5: top_n_total_qty={} (expected 100)", stats.top_n_total_qty);
        
        // Test with top_n = 20 (larger than available levels)
        let mut book2 = LocalL2Book::with_full_config(
            100,
            20,  // top_n = 20, but only 10 levels available
            0.01,
            CostEstimatorConfig::default()
        );
        
        // Add only 8 levels on each side
        for i in 0..8 {
            book2.update(Side::Buy, 100.0 - i as f64 * 0.1, 10.0);
            book2.update(Side::Sell, 100.1 + i as f64 * 0.1, 10.0);
        }
        
        let stats2 = book2.best();
        
        // Should only sum available 8 levels per side
        // 8 levels * 10 qty * 2 sides = 160
        assert!((stats2.top_n_total_qty - 160.0).abs() < 0.01);
        
        println!("top_n=20 with 8 levels: top_n_total_qty={} (expected 160)", stats2.top_n_total_qty);
    }
}
