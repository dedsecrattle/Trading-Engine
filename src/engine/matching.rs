use crate::engine::order::Side;
use crate::engine::order_book::OrderBook;
use crate::engine::trade::Trade;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct OrderRef {
    pub side: Side,
    pub price: u64,
}

#[derive(Debug, Default)]

pub struct MatchingEngine {
    pub order_book: OrderBook,
    pub trades: Vec<Trade>,
    pub next_trade_id: u64,
    pub active_orders: HashMap<u64, OrderRef>,
}
