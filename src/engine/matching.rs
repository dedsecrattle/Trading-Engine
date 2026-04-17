use crate::engine::order::{Order, Side};
use crate::engine::order_book::OrderBook;
use crate::engine::trade::Trade;

#[derive(Debug, Default)]
pub struct MatchingEngine {
    order_book: OrderBook,
    trades: Vec<Trade>,
    next_trade_id: u64,
    next_order_id: u64,
}

impl MatchingEngine {
    pub fn new() -> MatchingEngine {
        MatchingEngine {
            order_book: OrderBook::new(),
            trades: Vec::new(),
            next_trade_id: 1,
            next_order_id: 1,
        }
    }

    fn assign_order_id(&mut self, order: &mut Order) {
        order.id = self.next_order_id;
        self.next_order_id += 1;
    }

    pub fn submit_limit_order(&mut self, mut order: Order) -> Result<Vec<Trade>, String> {
        order.validate_new_limit()?;
        self.assign_order_id(&mut order);

        let (new_trades, next_tid) = match order.side {
            Side::Buy => {
                let limit = order.price.expect("validated limit buy");
                self.order_book
                    .match_incoming_buy(&mut order, limit, self.next_trade_id)
            }
            Side::Sell => {
                let limit = order.price.expect("validated limit sell");
                self.order_book
                    .match_incoming_sell(&mut order, limit, self.next_trade_id)
            }
        };
        self.next_trade_id = next_tid;

        if order.remaining > 0 {
            self.order_book.insert_resting_limit(order);
        }

        self.trades.extend(new_trades.iter().cloned());
        Ok(new_trades)
    }

    pub fn submit_market_order(&mut self, mut order: Order) -> Result<Vec<Trade>, String> {
        order.validate_new_market()?;
        self.assign_order_id(&mut order);

        let (new_trades, next_tid) = match order.side {
            Side::Buy => {
                self.order_book
                    .match_incoming_buy(&mut order, u64::MAX, self.next_trade_id)
            }
            Side::Sell => self
                .order_book
                .match_incoming_sell(&mut order, 0, self.next_trade_id),
        };
        self.next_trade_id = next_tid;

        self.trades.extend(new_trades.iter().cloned());
        Ok(new_trades)
    }

    pub fn trades(&self) -> &[Trade] {
        &self.trades
    }

    pub fn order_book(&self) -> &OrderBook {
        &self.order_book
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::order::OrderType;
    use chrono::Utc;

    fn mk_order(side: Side, order_type: OrderType, price: Option<u64>, qty: u64) -> Order {
        Order::new(side, order_type, price, qty, Utc::now())
    }

    #[test]
    fn limit_buy_matches_cheaper_ask_then_rest() {
        let mut eng = MatchingEngine::new();
        eng.submit_limit_order(mk_order(Side::Sell, OrderType::Limit, Some(100), 5))
            .unwrap();
        let trades = eng
            .submit_limit_order(mk_order(Side::Buy, OrderType::Limit, Some(100), 10))
            .unwrap();
        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].quantity, 5);
        assert_eq!(trades[0].price, 100);
        assert_eq!(trades[0].sell_order_id, 1);
        assert_eq!(trades[0].buy_order_id, 2);
        let ob = eng.order_book();
        assert_eq!(ob.bids().len(), 1);
        assert!(ob.asks().is_empty());
    }

    #[test]
    fn limit_buy_does_not_cross_when_price_too_low() {
        let mut eng = MatchingEngine::new();
        eng.submit_limit_order(mk_order(Side::Sell, OrderType::Limit, Some(101), 5))
            .unwrap();
        eng.submit_limit_order(mk_order(Side::Buy, OrderType::Limit, Some(100), 10))
            .unwrap();
        let ob = eng.order_book();
        assert!(ob.bids().contains_key(&100));
        assert!(ob.asks().contains_key(&101));
    }

    #[test]
    fn market_buy_consumes_asks() {
        let mut eng = MatchingEngine::new();
        eng.submit_limit_order(mk_order(Side::Sell, OrderType::Limit, Some(50), 3))
            .unwrap();
        let trades = eng
            .submit_market_order(mk_order(Side::Buy, OrderType::Market, None, 10))
            .unwrap();
        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].quantity, 3);
    }
}
