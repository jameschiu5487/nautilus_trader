pub fn queue_ahead_qty(queue_ahead: f64) -> f64 {
    queue_ahead
}

pub fn distance_to_mid_ticks(price: f64, mid: f64, tick_size: f64) -> f64 {
    (price - mid).abs() / tick_size
}