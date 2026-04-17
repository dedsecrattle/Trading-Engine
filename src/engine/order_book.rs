use std::collections::{BTreeMap, VecDeque};

use crate::engine::order::Order;

#[derive(Debug, Clone)]
pub struct PriceLevel {
    pub price: u64,
    pub orders: VecDeque<Order>,
}

impl PriceLevel {
    pub fn get_front(&mut self) -> &mut Order {
        self.orders.front_mut().unwrap()
    }
}

#[derive(Debug, Default)]
pub struct OrderBook {
    pub bids: BTreeMap<u64, PriceLevel>,
    pub asks: BTreeMap<u64, PriceLevel>,
}

impl OrderBook {
    pub fn new() -> OrderBook {
        OrderBook {
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
        }
    }
    pub fn best_bid(&mut self) -> Option<&mut PriceLevel> {
        self.bids.iter_mut().next_back().map(|(_, level)| level)
    }

    pub fn best_ask(&mut self) -> Option<&mut PriceLevel> {
        self.asks.iter_mut().next().map(|(_, level)| level)
    }
}
