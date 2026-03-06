use crate::book::LocalL2Book;
use crate::types::BookView;

/// Calculate gap between bid level 1 and bid level 2 (in ticks)
pub fn gap_ticks_bid(book: &LocalL2Book) -> f64 {
    book.best().gap_ticks_bid
}

/// Calculate gap between ask level 1 and ask level 2 (in ticks)
pub fn gap_ticks_ask(book: &LocalL2Book) -> f64 {
    book.best().gap_ticks_ask
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Side;
    
    #[test]
    fn test_gap_ticks() {
        let mut book = LocalL2Book::new(50, 0.01);
        book.update(Side::Buy, 100.0, 100.0);
        book.update(Side::Buy, 99.95, 50.0);
        book.update(Side::Sell, 100.10, 80.0);
        book.update(Side::Sell, 100.15, 60.0);
        
        let gap_bid = gap_ticks_bid(&book);
        let gap_ask = gap_ticks_ask(&book);
        
        assert!(gap_bid > 0.0);
        assert!(gap_ask > 0.0);
    }
}
