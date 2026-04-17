use std::collections::{BTreeMap, VecDeque};

use crate::engine::order::Order;

#[derive(Debug, Clone)]
pub struct PriceLevel {
    pub price: u64,
    pub orders: VecDeque<Order>,
}

#[derive(Debug, Default)]
pub struct OrderBook {
    pub bids: BTreeMap<u64, PriceLevel>,

    pub asks: BTreeMap<u64, PriceLevel>,
}
