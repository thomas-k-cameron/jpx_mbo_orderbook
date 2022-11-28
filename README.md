# Overview
This library reconstructs orderbook from itch procotocol message.
It is intended to be used with Osaka Exchange's market by order data (全注文情報), Osaka Exchange's proprietary dataset for recreating orderbook.

# How to use
1. Implement `OrderBookRunTimeCallback` trait.
2. prepare datasets
3. run it with `order_book_runtime`

# You must be aware that...
- It is barely optimized
- 