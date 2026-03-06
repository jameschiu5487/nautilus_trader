pub mod position;
pub mod service;
pub mod competition;

pub use position::{queue_ahead_qty, distance_to_mid_ticks};
pub use service::{est_service_rate, time_to_fill_est_ms};
pub use competition::qci;

#[derive(Debug, Clone)]
pub struct QueueFeatures {
    pub queue_ahead_qty: f64,
    pub distance_to_mid_ticks: f64,
    pub est_service_rate: f64,
    pub time_to_fill_est_ms: f64,
    pub qci: f64,
}

impl QueueFeatures {

    pub fn default_zero() -> Self {
        Self {
            queue_ahead_qty: 0.0,
            distance_to_mid_ticks: 0.0,
            est_service_rate: 0.0,
            time_to_fill_est_ms: 0.0,
            qci: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_queue_features_creation() {
        let features = QueueFeatures::default_zero();
        assert_eq!(features.queue_ahead_qty, 0.0);
        assert_eq!(features.qci, 0.0);
    }
}
