pub fn cost_per_min_bps(cost: f64) -> f64 {
    cost
}

pub fn cost_per_day_bps(cost_per_min: f64) -> f64 {
    cost_per_min * 1440.0
}
