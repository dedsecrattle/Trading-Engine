use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct Trade {
    pub trade_id: u64,
    pub buy_order_id: u64,
    pub sell_order_id: u64,
    pub price: u64,
    pub quantity: u64,
    pub timestamp: DateTime<Utc>,
}

impl Trade {
    pub fn new(
        trade_id: u64,
        buy_order_id: u64,
        sell_order_id: u64,
        price: u64,
        quantity: u64,
    ) -> Self {
        Self {
            trade_id,
            buy_order_id,
            sell_order_id,
            price,
            quantity,
            timestamp: Utc::now(),
        }
    }
}
