use chrono::Utc;
use std::collections::VecDeque;

use crate::engine::order::{Order, OrderType, Side};
use crate::engine::order_book::{OrderBook, PriceLevel};
use crate::engine::trade::Trade;

#[derive(Debug, Default)]
pub struct MatchingEngine {
    pub order_book: OrderBook,
    pub trades: Vec<Trade>,
    pub next_trade_id: u64,
    pub next_order_id: u64,
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

    fn validate_limit_order(&self, order: &Order) -> Result<(), String> {
        if order.order_type != OrderType::Limit {
            return Err("submit_limit_order requires OrderType::Limit".to_string());
        }
        if order.quantity == 0 {
            return Err("quantity must be greater than 0".to_string());
        }
        if order.remaining != order.quantity {
            return Err("remaining must equal quantity for new orders".to_string());
        }
        let price = order.price.ok_or("limit order must have a price")?;
        if price == 0 {
            return Err("price must be greater than 0".to_string());
        }
        Ok(())
    }

    fn validate_market_order(&self, order: &Order) -> Result<(), String> {
        if order.order_type != OrderType::Market {
            return Err("submit_market_order requires OrderType::Market".to_string());
        }
        if order.quantity == 0 {
            return Err("quantity must be greater than 0".to_string());
        }
        if order.remaining != order.quantity {
            return Err("remaining must equal quantity for new orders".to_string());
        }
        if order.price.is_some() {
            return Err("market order must not have a price".to_string());
        }
        Ok(())
    }

    /// Match a buy (market or limit) against resting sells. `max_ask_price` is inclusive:
    /// stops when the best ask exceeds this (use `u64::MAX` for a market buy).
    fn match_incoming_buy(&mut self, incoming: &mut Order, max_ask_price: u64) -> Vec<Trade> {
        let mut trades = Vec::new();

        while incoming.remaining > 0 {
            let Some(ask_price) = self.order_book.asks.keys().next().copied() else {
                break;
            };
            if ask_price > max_ask_price {
                break;
            }

            let trade_id = self.next_trade_id;
            self.next_trade_id += 1;

            let level_empty = {
                let level = self
                    .order_book
                    .asks
                    .get_mut(&ask_price)
                    .expect("ask level must exist");
                let resting = level.get_front();
                let trade_qty = incoming.remaining.min(resting.remaining);

                trades.push(Trade {
                    trade_id,
                    buy_order_id: incoming.id,
                    sell_order_id: resting.id,
                    price: ask_price,
                    quantity: trade_qty,
                    timestamp: Utc::now(),
                });

                incoming.remaining -= trade_qty;
                resting.remaining -= trade_qty;

                if resting.remaining == 0 {
                    level.orders.pop_front();
                }
                level.orders.is_empty()
            };

            if level_empty {
                self.order_book.asks.remove(&ask_price);
            }
        }

        trades
    }

    /// Match a sell (market or limit) against resting buys. `min_bid_price` is inclusive:
    /// stops when the best bid is below this (use `0` for a market sell).
    fn match_incoming_sell(&mut self, incoming: &mut Order, min_bid_price: u64) -> Vec<Trade> {
        let mut trades = Vec::new();

        while incoming.remaining > 0 {
            let Some(bid_price) = self.order_book.bids.keys().next_back().copied() else {
                break;
            };
            if bid_price < min_bid_price {
                break;
            }

            let trade_id = self.next_trade_id;
            self.next_trade_id += 1;

            let level_empty = {
                let level = self
                    .order_book
                    .bids
                    .get_mut(&bid_price)
                    .expect("bid level must exist");
                let resting = level.get_front();
                let trade_qty = incoming.remaining.min(resting.remaining);

                trades.push(Trade {
                    trade_id,
                    buy_order_id: resting.id,
                    sell_order_id: incoming.id,
                    price: bid_price,
                    quantity: trade_qty,
                    timestamp: Utc::now(),
                });

                incoming.remaining -= trade_qty;
                resting.remaining -= trade_qty;

                if resting.remaining == 0 {
                    level.orders.pop_front();
                }
                level.orders.is_empty()
            };

            if level_empty {
                self.order_book.bids.remove(&bid_price);
            }
        }

        trades
    }

    fn match_buy_order(&mut self, incoming: &mut Order) -> Vec<Trade> {
        let limit = incoming.price.expect("validated limit buy");
        self.match_incoming_buy(incoming, limit)
    }

    fn match_sell_order(&mut self, incoming: &mut Order) -> Vec<Trade> {
        let limit = incoming.price.expect("validated limit sell");
        self.match_incoming_sell(incoming, limit)
    }

    fn match_market_buy_order(&mut self, incoming: &mut Order) -> Vec<Trade> {
        self.match_incoming_buy(incoming, u64::MAX)
    }

    fn match_market_sell_order(&mut self, incoming: &mut Order) -> Vec<Trade> {
        self.match_incoming_sell(incoming, 0)
    }

    fn insert_resting_order(&mut self, order: Order) {
        let price = order.price.expect("resting limit order must have price");
        let level = match order.side {
            Side::Buy => self
                .order_book
                .bids
                .entry(price)
                .or_insert_with(|| PriceLevel {
                    price,
                    orders: VecDeque::new(),
                }),
            Side::Sell => self
                .order_book
                .asks
                .entry(price)
                .or_insert_with(|| PriceLevel {
                    price,
                    orders: VecDeque::new(),
                }),
        };
        level.orders.push_back(order);
    }

    pub fn submit_limit_order(&mut self, mut order: Order) -> Result<Vec<Trade>, String> {
        self.validate_limit_order(&order)?;
        self.assign_order_id(&mut order);

        let new_trades = match order.side {
            Side::Buy => self.match_buy_order(&mut order),
            Side::Sell => self.match_sell_order(&mut order),
        };

        if order.remaining > 0 {
            self.insert_resting_order(order);
        }

        self.trades.extend(new_trades.iter().cloned());
        Ok(new_trades)
    }

    pub fn submit_market_order(&mut self, mut order: Order) -> Result<Vec<Trade>, String> {
        self.validate_market_order(&order)?;
        self.assign_order_id(&mut order);

        let new_trades = match order.side {
            Side::Buy => self.match_market_buy_order(&mut order),
            Side::Sell => self.match_market_sell_order(&mut order),
        };

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
        assert_eq!(ob.bids.len(), 1);
        assert!(ob.asks.is_empty());
    }

    #[test]
    fn limit_buy_does_not_cross_when_price_too_low() {
        let mut eng = MatchingEngine::new();
        eng.submit_limit_order(mk_order(Side::Sell, OrderType::Limit, Some(101), 5))
            .unwrap();
        eng.submit_limit_order(mk_order(Side::Buy, OrderType::Limit, Some(100), 10))
            .unwrap();
        assert!(eng.order_book().bids.contains_key(&100));
        assert!(eng.order_book().asks.contains_key(&101));
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
