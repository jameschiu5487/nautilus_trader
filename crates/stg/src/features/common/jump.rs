use std::collections::VecDeque;

/// Milliseconds to nanoseconds conversion
const MS_TO_NS: u64 = 1_000_000;

/// Price tick data point
#[derive(Debug, Clone, Copy)]
pub struct PriceTick {
    pub timestamp_ns: u64,
    pub price: f64,
}

#[derive(Debug, Clone, Copy)]
struct MoveEvent {
    timestamp_ns: u64,
    move_bps: f64,
}

pub struct JumpCalculator {
    window_ns: u64,
    threshold_bps: f64,
    prices: VecDeque<PriceTick>,
    move_events: VecDeque<MoveEvent>,
    max_queue: VecDeque<MoveEvent>,
}

impl JumpCalculator {
    /// # Arguments
    /// * `window_ms` - Window size in milliseconds (e.g., 500, 1000, 2000)
    /// * `threshold_bps` - Jump threshold in basis points (default: 50 bps)
    pub fn new(window_ms: u64, threshold_bps: f64) -> Self {
        Self {
            window_ns: window_ms * MS_TO_NS,
            threshold_bps,
            prices: VecDeque::new(),
            move_events: VecDeque::new(),
            max_queue: VecDeque::new(),
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
            let move_bps = ((price - last_tick.price) / last_tick.price).abs() * 10_000.0;
            
            let move_event = MoveEvent {
                timestamp_ns,
                move_bps,
            };
            
            self.move_events.push_back(move_event);
            
            while let Some(back) = self.max_queue.back() {
                if back.move_bps <= move_bps {
                    self.max_queue.pop_back();
                } else {
                    break;
                }
            }
            
            self.max_queue.push_back(move_event);
        }
        
        self.prices.push_back(PriceTick {
            timestamp_ns,
            price,
        });
        
        self.clean_old_data(timestamp_ns);
    }
    pub fn jump_count(&self) -> usize {
        self.move_events
            .iter()
            .filter(|e| e.move_bps > self.threshold_bps)
            .count()
    }
    
    pub fn max_move_bps(&self) -> f64 {
        self.max_queue.front().map(|e| e.move_bps).unwrap_or(0.0)
    }
    
    fn clean_old_data(&mut self, latest_timestamp_ns: u64) {
        let cutoff = latest_timestamp_ns.saturating_sub(self.window_ns);
        
        while let Some(tick) = self.prices.front() {
            if tick.timestamp_ns < cutoff {
                self.prices.pop_front();
            } else {
                break;
            }
        }
        
        while let Some(event) = self.move_events.front() {
            if event.timestamp_ns < cutoff {
                self.move_events.pop_front();
            } else {
                break;
            }
        }
        
        while let Some(front) = self.max_queue.front() {
            if front.timestamp_ns < cutoff {
                self.max_queue.pop_front();
            } else {
                break;
            }
        }
    }
}

impl Default for JumpCalculator {
    /// Default: 1s window with 50 bps threshold
    fn default() -> Self {
        Self::new(1000, 50.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_jump_calculator_creation() {
        let calc = JumpCalculator::new(1000, 50.0);
        assert_eq!(calc.window_ms(), 1000);
        assert_eq!(calc.jump_count(), 0);
    }
    
    #[test]
    fn test_no_jump() {
        let mut calc = JumpCalculator::new(1000, 50.0);
        calc.update(1000, 100.0);
        calc.update(2000, 100.1);
        
        assert_eq!(calc.jump_count(), 0);
    }
    
    #[test]
    fn test_single_jump() {
        let mut calc = JumpCalculator::new(1000, 50.0);
        calc.update(1000, 100.0);
        calc.update(2000, 101.0);  // 1% move = 100 bps > 50 bps
        
        assert_eq!(calc.jump_count(), 1);
    }
    
    #[test]
    fn test_max_move_bps() {
        let mut calc = JumpCalculator::new(1000, 50.0);
        calc.update(1000, 100.0);
        calc.update(2000, 101.0);
        
        let max_move = calc.max_move_bps();
        assert!((max_move - 100.0).abs() < 1e-6);
    }
    
    #[test]
    fn test_custom_window_2s() {
        let mut calc = JumpCalculator::new(2000, 50.0);
        assert_eq!(calc.window_ms(), 2000);
        
        calc.update(1000, 100.0);
        calc.update(2000, 101.0);
        assert_eq!(calc.jump_count(), 1);
    }
    
    #[test]
    fn test_monotonic_queue_maintains_max() {
        let mut calc = JumpCalculator::new(5000, 50.0);
        
        // Add moves
        calc.update(1000, 100.0);
        calc.update(2000, 101.0);  // move = (101-100)/100 * 10000 = 100 bps
        assert!((calc.max_move_bps() - 100.0).abs() < 1e-6);
        
        calc.update(3000, 101.05); // move = (101.05-101)/101 * 10000 ≈ 49.5 bps
        assert!((calc.max_move_bps() - 100.0).abs() < 1e-6);  // max still 100
        
        calc.update(4000, 103.0);  // move = (103-101.05)/101.05 * 10000 ≈ 193 bps
        assert!(calc.max_move_bps() > 190.0);  // new max ~193
        
        calc.update(5000, 103.03); // move = (103.03-103)/103 * 10000 ≈ 29 bps
        assert!(calc.max_move_bps() > 190.0);  // max still ~193
    }
}
