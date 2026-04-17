use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderType {
    Limit,
    Market,
}

#[derive(Debug, Clone)]
pub struct Order {
    id: u64,
    side: Side,
    order_type: OrderType,
    price: Option<u64>,
    quantity: u64,
    remaining: u64,
    timestamp: DateTime<Utc>,
}

impl Order {
    pub fn new(
        side: Side,
        order_type: OrderType,
        price: Option<u64>,
        quantity: u64,
        timestamp: DateTime<Utc>,
    ) -> Order {
        Order {
            id: 1,
            side,
            order_type,
            price,
            quantity,
            timestamp,
            remaining: quantity,
        }
    }
}
