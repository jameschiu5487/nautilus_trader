use crate::types::Side;
use std::collections::VecDeque;
const MS_TO_NS: u64 = 1_000_000;

#[derive(Debug, Clone, Copy)]
pub struct OrderEvent {
    pub timestamp_ns: u64,
    pub side: Side,
    pub price: f64,
    pub size: f64,
    pub is_cancel: bool,
}

pub struct OfiCalculator {
    window_ns: u64,
    order_events: VecDeque<OrderEvent>,
    ofi_sum: f64,
    cancel_count: usize,
    total_count: usize,
}

impl OfiCalculator {
    /// # Arguments
    /// * `window_ms` - Window size in milliseconds (e.g., 200, 300, 500, 1000)
    /// 
    /// # Examples
    /// ```
    /// use nautilus_stg::OfiCalculator;
    /// 
    /// let ofi_200ms = OfiCalculator::new(200);  // 200ms window
    /// let ofi_300ms = OfiCalculator::new(300);  // 300ms window
    /// let ofi_1s = OfiCalculator::new(1000);    // 1s window
    /// ```
    pub fn new(window_ms: u64) -> Self {
        Self {
            window_ns: window_ms * MS_TO_NS,
            order_events: VecDeque::new(),
            ofi_sum: 0.0,
            cancel_count: 0,
            total_count: 0,
        }
    }
    
    pub fn window_ms(&self) -> u64 {
        self.window_ns / MS_TO_NS
    }
    
    pub fn add_order_event(
        &mut self,
        timestamp_ns: u64,
        side: Side,
        price: f64,
        size: f64,
        is_cancel: bool,
    ) {
        let event = OrderEvent {
            timestamp_ns,
            side,
            price,
            size,
            is_cancel,
        };
        
        let sign = match event.side {
            Side::Buy => 1.0,
            Side::Sell => -1.0,
        };
        let ofi_delta = if event.is_cancel { 
            -sign * event.size 
        } else { 
            sign * event.size 
        };
        
        self.ofi_sum += ofi_delta;
        self.total_count += 1;
        if event.is_cancel {
            self.cancel_count += 1;
        }
        
        self.order_events.push_back(event);
        self.clean_old_events(timestamp_ns);
    }
    
    pub fn ofi(&self) -> f64 {
        self.ofi_sum
    }
    
    pub fn cancel_ratio(&self) -> f64 {
        if self.total_count == 0 {
            0.0
        } else {
            self.cancel_count as f64 / self.total_count as f64
        }
    }
    
    fn clean_old_events(&mut self, latest_timestamp_ns: u64) {
        let cutoff = latest_timestamp_ns.saturating_sub(self.window_ns);
        
        while let Some(event) = self.order_events.front() {
            if event.timestamp_ns < cutoff {
                let event = self.order_events.pop_front().unwrap();
                let sign = match event.side {
                    Side::Buy => 1.0,
                    Side::Sell => -1.0,
                };
                let ofi_delta = if event.is_cancel { 
                    -sign * event.size 
                } else { 
                    sign * event.size 
                };
                
                self.ofi_sum -= ofi_delta;
                self.total_count -= 1;
                if event.is_cancel {
                    self.cancel_count -= 1;
                }
            } else {
                break;
            }
        }
    }
}

impl Default for OfiCalculator {
    /// Default to 1s window
    fn default() -> Self {
        Self::new(1000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ofi_calculator_creation() {
        let calc = OfiCalculator::new(200);
        assert_eq!(calc.window_ms(), 200);
        assert_eq!(calc.ofi(), 0.0);
    }
    
    #[test]
    fn test_ofi_buy_order() {
        let mut calc = OfiCalculator::new(200);
        calc.add_order_event(1000, Side::Buy, 100.0, 10.0, false);
        
        assert_eq!(calc.ofi(), 10.0);
    }
    
    #[test]
    fn test_ofi_sell_order() {
        let mut calc = OfiCalculator::new(200);
        calc.add_order_event(1000, Side::Sell, 100.10, 5.0, false);
        
        assert_eq!(calc.ofi(), -5.0);
    }
    
    #[test]
    fn test_ofi_mixed_orders() {
        let mut calc = OfiCalculator::new(1000);
        calc.add_order_event(1000, Side::Buy, 100.0, 10.0, false);
        calc.add_order_event(2000, Side::Sell, 100.10, 5.0, false);
        
        // (+1 * 10) + (-1 * 5) = 5.0
        assert_eq!(calc.ofi(), 5.0);
    }
    
    #[test]
    fn test_cancel_ratio() {
        let mut calc = OfiCalculator::new(1000);
        calc.add_order_event(1000, Side::Buy, 100.0, 10.0, false);
        calc.add_order_event(2000, Side::Buy, 100.0, 5.0, true);  // cancel
        
        assert_eq!(calc.cancel_ratio(), 0.5); // 1 cancel out of 2 total
    }
    
    #[test]
    fn test_custom_window_300ms() {
        let mut calc = OfiCalculator::new(300);
        assert_eq!(calc.window_ms(), 300);
        
        calc.add_order_event(1000, Side::Buy, 100.0, 10.0, false);
        assert_eq!(calc.ofi(), 10.0);
    }
    
    #[test]
    fn test_window_sliding() {
        let mut calc = OfiCalculator::new(1000);  // 1s = 1,000,000,000 ns
        
        // Add events at different times
        calc.add_order_event(1_000_000_000, Side::Buy, 100.0, 10.0, false);
        assert_eq!(calc.ofi(), 10.0);
        assert_eq!(calc.total_count, 1);
        
        calc.add_order_event(1_500_000_000, Side::Sell, 100.0, 5.0, false);
        assert_eq!(calc.ofi(), 5.0);  // 10 - 5
        assert_eq!(calc.total_count, 2);
        
        // Add event that expires the first one (> 1s later)
        calc.add_order_event(2_100_000_000, Side::Buy, 100.0, 3.0, false);
        // First event (10.0) should be removed
        assert_eq!(calc.ofi(), -2.0);  // -5 + 3
        assert_eq!(calc.total_count, 2);
    }
}
