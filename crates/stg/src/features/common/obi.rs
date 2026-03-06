use crate::book::LocalL2Book;
use crate::types::{Side, BookView};

pub fn obi_n(book: &LocalL2Book, n: usize) -> f64 {
    let (_bid_prices, bid_sizes) = book.top_k(Side::Buy, n);
    let (_ask_prices, ask_sizes) = book.top_k(Side::Sell, n);
    
    let bid_total: f64 = bid_sizes.iter().sum();
    let ask_total: f64 = ask_sizes.iter().sum();
    
    if bid_total + ask_total > 0.0 {
        (bid_total - ask_total) / (bid_total + ask_total)
    } else {
        0.0
    }
}

pub fn top_n_total_qty(book: &LocalL2Book) -> f64 {
    book.best().top_n_total_qty
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_obi_balanced() {
        let mut book = LocalL2Book::new(50, 0.01);
        book.update(Side::Buy, 100.0, 100.0);
        book.update(Side::Sell, 100.10, 100.0);
        
        let obi = obi_n(&book, 1);
        assert_eq!(obi, 0.0); // Balanced
    }
    
    #[test]
    fn test_obi_bid_heavy() {
        let mut book = LocalL2Book::new(50, 0.01);
        book.update(Side::Buy, 100.0, 200.0);
        book.update(Side::Sell, 100.10, 100.0);
        
        let obi = obi_n(&book, 1);
        // (200 - 100) / (200 + 100) = 100/300 = 0.333...
        assert!(obi > 0.3 && obi < 0.4);
    }
    
    #[test]
    fn test_top_n_total_qty() {
        let mut book = LocalL2Book::new(50, 0.01);
        book.update(Side::Buy, 100.0, 100.0);
        book.update(Side::Sell, 100.10, 80.0);
        
        let total = top_n_total_qty(&book);
        assert!(total > 0.0);
    }
}
