pub fn est_service_rate(avg_trade_qty: f64, trade_count: f64) -> f64 {
    if trade_count > 0.0 {
        avg_trade_qty / (1000.0 / trade_count.max(1.0))
    } else {
        0.01 // Default minimum rate
    }
}

pub fn time_to_fill_est_ms(queue_ahead: f64, service_rate: f64) -> f64 {
    if service_rate > 0.0 {
        queue_ahead / service_rate
    } else {
        f64::INFINITY
    }
}