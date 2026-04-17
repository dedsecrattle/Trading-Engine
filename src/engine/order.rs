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
    pub id: u64,
    pub side: Side,
    pub order_type: OrderType,
    pub price: Option<u64>,
    pub quantity: u64,
    pub remaining: u64,
    pub timestamp: DateTime<Utc>,
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

    /// Validates an order before it is assigned an id and matched (new limit submission).
    pub fn validate_new_limit(&self) -> Result<(), String> {
        if self.order_type != OrderType::Limit {
            return Err("submit_limit_order requires OrderType::Limit".to_string());
        }
        if self.quantity == 0 {
            return Err("quantity must be greater than 0".to_string());
        }
        if self.remaining != self.quantity {
            return Err("remaining must equal quantity for new orders".to_string());
        }
        let price = self.price.ok_or("limit order must have a price")?;
        if price == 0 {
            return Err("price must be greater than 0".to_string());
        }
        Ok(())
    }

    /// Validates an order before it is assigned an id and matched (new market submission).
    pub fn validate_new_market(&self) -> Result<(), String> {
        if self.order_type != OrderType::Market {
            return Err("submit_market_order requires OrderType::Market".to_string());
        }
        if self.quantity == 0 {
            return Err("quantity must be greater than 0".to_string());
        }
        if self.remaining != self.quantity {
            return Err("remaining must equal quantity for new orders".to_string());
        }
        if self.price.is_some() {
            return Err("market order must not have a price".to_string());
        }
        Ok(())
    }
}
