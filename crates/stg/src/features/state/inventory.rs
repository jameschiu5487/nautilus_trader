pub fn inv_skew(inventory: f64, target: f64, inventory_max: f64) -> f64 {
    if inventory_max > 0.0 {
        (inventory - target) / inventory_max
    } else {
        0.0
    }
}

pub fn exposure_notional(inventory: f64, mid_price: f64) -> f64 {
    inventory * mid_price
}
