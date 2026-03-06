pub fn rate_limit_risk_level(usage: f64) -> f64 {
    if usage > 0.8 {
        (0.9 + (usage - 0.8) * 0.5).min(1.0)
    } else {
        usage
    }
}