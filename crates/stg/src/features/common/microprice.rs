
use crate::book::LocalL2Book;
use crate::types::BookView;

pub fn microprice(book: &LocalL2Book) -> f64 {
    book.microprice().unwrap_or(book.best().mid)
}

pub fn mp_minus_mid_bps(book: &LocalL2Book) -> f64 {
    book.microprice_deviation_bps().unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Side;
    
    #[test]
    fn test_microprice() {
        let mut book = LocalL2Book::new(50, 0.01);
        book.update(Side::Buy, 100.0, 100.0);
        book.update(Side::Sell, 100.10, 80.0);
        
        let mp = microprice(&book);
        assert!(mp > 0.0);
        assert!(mp >= 100.0 && mp <= 100.10);
    }
    
    #[test]
    fn test_mp_deviation() {
        let mut book = LocalL2Book::new(50, 0.01);
        book.update(Side::Buy, 100.0, 100.0);
        book.update(Side::Sell, 100.10, 80.0);
        
        let dev = mp_minus_mid_bps(&book);
        // Should be small since book is relatively balanced
        assert!(dev.abs() < 50.0);
    }
}
