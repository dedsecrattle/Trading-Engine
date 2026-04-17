use chrono::Utc;

mod engine;
use engine::order::{Order, OrderType, Side};

fn main() {
    let buy_order = Order::new(Side::Buy, OrderType::Limit, Some(100), 10, Utc::now());
    println!("{:?}", buy_order)
}
