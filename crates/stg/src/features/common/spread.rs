use crate::book::LocalL2Book;
use crate::types::BookView;

pub fn spread_ticks(book: &LocalL2Book) -> f64 {
    book.best().spread_ticks
}

pub fn spread_bps(book: &LocalL2Book) -> f64 {
    book.best().spread_bps
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Side;
    
    #[test]
    fn test_spread_ticks() {
        let mut book = LocalL2Book::new(50, 0.01);
        book.update(Side::Buy, 100.0, 100.0);
        book.update(Side::Sell, 100.10, 80.0);
        
        let spread = spread_ticks(&book);
        assert!((spread - 10.0).abs() < 1e-6); // ~10 ticks
    }
    
    #[test]
    fn test_spread_bps() {
        let mut book = LocalL2Book::new(50, 0.01);
        book.update(Side::Buy, 100.0, 100.0);
        book.update(Side::Sell, 100.10, 80.0);
        
        let spread = spread_bps(&book);
        assert!(spread > 0.0);
        assert!(spread < 200.0); // ~10 bps
    }
}
