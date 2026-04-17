# Trading Engine

A small in-memory **limit-order-book** and **matching engine** in Rust: resting orders, price-time priority at each level, limit and market orders, and trade capture with timestamps.

## Quick start

```bash
cargo run    # demo scenario + order book / trade ledger output
cargo test   # unit tests for matching
```

## What it does

- **Limit orders** — validated, assigned monotonic order IDs, matched against the opposite side when prices cross; any unfilled quantity rests on the book.
- **Market orders** — consume liquidity at the best available prices until filled or the book is empty (no resting quantity for pure market orders).
- **Book structure** — bids and asks as `BTreeMap` price levels; each level is a FIFO queue (`VecDeque`) of orders.
- **Trades** — each fill gets a trade ID, size, price, maker/taker order IDs, and an execution timestamp (`chrono`).

## Project layout

| Path | Role |
|------|------|
| `src/engine/order.rs` | `Order`, `Side`, `OrderType`, validation for new submissions |
| `src/engine/order_book.rs` | `OrderBook`, `PriceLevel`; matching and resting insertion |
| `src/engine/trade.rs` | `Trade` and constructor |
| `src/engine/matching.rs` | `MatchingEngine` — IDs, delegates to `OrderBook`, stores trade history |
| `src/main.rs` | Example run: multi-level matching, market sell, timestamped prints |

## Dependencies

- **[chrono](https://crates.io/crates/chrono)** — `DateTime<Utc>` on orders and trades (serde feature enabled for future serialization).

## Scope

This is a **single-process, educational** matcher: no networking, persistence, or instrument/multi-market routing. It is a reasonable base to extend toward a fuller exchange stack.
