use chrono::{DateTime, SecondsFormat, Utc};

mod engine;
use engine::matching::MatchingEngine;
use engine::order::{Order, OrderType, Side};

fn fmt_time(dt: DateTime<Utc>) -> String {
    dt.to_rfc3339_opts(SecondsFormat::Millis, true)
}

fn print_order_book(engine: &MatchingEngine) {
    let ob = engine.order_book();
    println!("\n--- order book ---");
    println!("asks (lowest first):");
    for (price, level) in ob.asks().iter() {
        for o in level.orders() {
            println!(
                "  @{}  sell  order {}  remaining {}  submitted {}",
                price,
                o.id,
                o.remaining,
                fmt_time(o.timestamp)
            );
        }
    }
    if ob.asks().is_empty() {
        println!("  (empty)");
    }
    println!("bids (highest first):");
    for (price, level) in ob.bids().iter().rev() {
        for o in level.orders() {
            println!(
                "  @{}  buy   order {}  remaining {}  submitted {}",
                price,
                o.id,
                o.remaining,
                fmt_time(o.timestamp)
            );
        }
    }
    if ob.bids().is_empty() {
        println!("  (empty)");
    }
}

fn print_trade_ledger(engine: &MatchingEngine) {
    println!("\n--- trade ledger (all fills) ---");
    for t in engine.trades() {
        println!(
            "  trade {}: {} @ {}  buy #{}  sell #{}  executed {}",
            t.trade_id,
            t.quantity,
            t.price,
            t.buy_order_id,
            t.sell_order_id,
            fmt_time(t.timestamp)
        );
    }
    if engine.trades().is_empty() {
        println!("  (none)");
    }
}

fn main() {
    let mut engine = MatchingEngine::new();

    println!("1) Rest liquidity: sell 15 @ 100, sell 10 @ 101");
    engine
        .submit_limit_order(Order::new(
            Side::Sell,
            OrderType::Limit,
            Some(100),
            15,
            Utc::now(),
        ))
        .expect("sell 100");
    engine
        .submit_limit_order(Order::new(
            Side::Sell,
            OrderType::Limit,
            Some(101),
            10,
            Utc::now(),
        ))
        .expect("sell 101");

    println!("\n2) Aggressive limit buy 20 @ 102 — walks both ask levels");
    let t1 = engine
        .submit_limit_order(Order::new(
            Side::Buy,
            OrderType::Limit,
            Some(102),
            20,
            Utc::now(),
        ))
        .expect("buy");
    for t in &t1 {
        println!(
            "   trade {}: {} @ {} (buy #{} vs sell #{})  {}",
            t.trade_id,
            t.quantity,
            t.price,
            t.buy_order_id,
            t.sell_order_id,
            fmt_time(t.timestamp)
        );
    }

    println!("\n3) Rest a bid 8 @ 99, then market sell 5 — hits best bid");
    engine
        .submit_limit_order(Order::new(
            Side::Buy,
            OrderType::Limit,
            Some(99),
            8,
            Utc::now(),
        ))
        .expect("bid 99");
    let t2 = engine
        .submit_market_order(Order::new(
            Side::Sell,
            OrderType::Market,
            None,
            5,
            Utc::now(),
        ))
        .expect("market sell");
    for t in &t2 {
        println!(
            "   trade {}: {} @ {} (buy #{} vs sell #{})  {}",
            t.trade_id,
            t.quantity,
            t.price,
            t.buy_order_id,
            t.sell_order_id,
            fmt_time(t.timestamp)
        );
    }

    println!(
        "\n4) Cumulative fills (engine.trades): {}",
        engine.trades().len()
    );
    print_trade_ledger(&engine);
    print_order_book(&engine);
}
