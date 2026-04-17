use std::collections::{BTreeMap, VecDeque};

use crate::engine::order::{Order, Side};
use crate::engine::trade::Trade;

#[derive(Debug, Clone)]
pub struct PriceLevel {
    price: u64,
    orders: VecDeque<Order>,
}

impl PriceLevel {
    pub fn new(price: u64) -> Self {
        Self {
            price,
            orders: VecDeque::new(),
        }
    }

    pub fn orders(&self) -> &VecDeque<Order> {
        &self.orders
    }

    pub fn is_empty(&self) -> bool {
        self.orders.is_empty()
    }

    pub fn push_back(&mut self, order: Order) {
        self.orders.push_back(order);
    }

    pub fn match_incoming_buy(&mut self, incoming: &mut Order, trade_id: u64) -> Option<Trade> {
        let resting = self.orders.front_mut()?;
        let trade_qty = incoming.remaining.min(resting.remaining);
        let sell_order_id = resting.id;
        let price = self.price;

        incoming.remaining -= trade_qty;
        resting.remaining -= trade_qty;

        let trade = Trade::new(trade_id, incoming.id, sell_order_id, price, trade_qty);

        if resting.remaining == 0 {
            self.orders.pop_front();
        }

        Some(trade)
    }

    pub fn match_incoming_sell(&mut self, incoming: &mut Order, trade_id: u64) -> Option<Trade> {
        let resting = self.orders.front_mut()?;
        let trade_qty = incoming.remaining.min(resting.remaining);
        let buy_order_id = resting.id;
        let price = self.price;

        incoming.remaining -= trade_qty;
        resting.remaining -= trade_qty;

        let trade = Trade::new(trade_id, buy_order_id, incoming.id, price, trade_qty);

        if resting.remaining == 0 {
            self.orders.pop_front();
        }

        Some(trade)
    }
}

#[derive(Debug, Default)]
pub struct OrderBook {
    bids: BTreeMap<u64, PriceLevel>,
    asks: BTreeMap<u64, PriceLevel>,
}

impl OrderBook {
    pub fn new() -> OrderBook {
        OrderBook {
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
        }
    }

    pub fn bids(&self) -> &BTreeMap<u64, PriceLevel> {
        &self.bids
    }

    pub fn asks(&self) -> &BTreeMap<u64, PriceLevel> {
        &self.asks
    }

    pub fn best_bid_price(&self) -> Option<u64> {
        self.bids.keys().next_back().copied()
    }

    pub fn best_ask_price(&self) -> Option<u64> {
        self.asks.keys().next().copied()
    }

    pub fn match_incoming_buy(
        &mut self,
        incoming: &mut Order,
        max_ask_price: u64,
        mut next_trade_id: u64,
    ) -> (Vec<Trade>, u64) {
        let mut trades = Vec::new();

        while incoming.remaining > 0 {
            let Some(ask_price) = self.best_ask_price() else {
                break;
            };
            if ask_price > max_ask_price {
                break;
            }

            let trade = {
                let level = self
                    .asks
                    .get_mut(&ask_price)
                    .expect("best ask price must exist in book");
                level
                    .match_incoming_buy(incoming, next_trade_id)
                    .expect("ask level with known price must have orders")
            };

            next_trade_id += 1;
            trades.push(trade);

            if self.asks.get(&ask_price).is_some_and(|lvl| lvl.is_empty()) {
                self.asks.remove(&ask_price);
            }
        }

        (trades, next_trade_id)
    }

    pub fn match_incoming_sell(
        &mut self,
        incoming: &mut Order,
        min_bid_price: u64,
        mut next_trade_id: u64,
    ) -> (Vec<Trade>, u64) {
        let mut trades = Vec::new();

        while incoming.remaining > 0 {
            let Some(bid_price) = self.best_bid_price() else {
                break;
            };
            if bid_price < min_bid_price {
                break;
            }

            let trade = {
                let level = self
                    .bids
                    .get_mut(&bid_price)
                    .expect("best bid price must exist in book");
                level
                    .match_incoming_sell(incoming, next_trade_id)
                    .expect("bid level with known price must have orders")
            };

            next_trade_id += 1;
            trades.push(trade);

            if self.bids.get(&bid_price).is_some_and(|lvl| lvl.is_empty()) {
                self.bids.remove(&bid_price);
            }
        }

        (trades, next_trade_id)
    }

    pub fn insert_resting_limit(&mut self, order: Order) {
        let price = order
            .price
            .expect("resting order passed to insert_resting_limit must have a price");
        match order.side {
            Side::Buy => {
                self.bids
                    .entry(price)
                    .or_insert_with(|| PriceLevel::new(price))
                    .push_back(order);
            }
            Side::Sell => {
                self.asks
                    .entry(price)
                    .or_insert_with(|| PriceLevel::new(price))
                    .push_back(order);
            }
        }
    }
}
