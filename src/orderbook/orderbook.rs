use crate::{
    CombinationProduct, DeleteOrder, EquilibriumPrice, Executed, ExecutionWithPriceInfo,
    ProductInfo, AddOrder, Side, TickSize, TradingStatusInfo,
};
use std::collections::{BTreeMap, HashMap};

#[derive(Debug)]
pub struct OrderBook {
    ///
    /// 銘柄基本情報
    pub product_info: ProductInfo,
    pub legs: Vec<CombinationProduct>,
    ///
    /// tick info
    pub tick_info: Vec<TickSize>,
    /// orders are identifiable with their id and side.
    /// Orders with same id could exists on the other side of the orderbook.
    /// index to map orders
    pub orders: HashMap<(u64, Side), i64>, // id => price
    /// key is the price, embeded map's key is the order id of the value's put order
    /// price => {id: AddOrder}
    pub ask: PriceLevel,
    /// key is the price, embeded map's key is the order id of the value's put order
    /// price => {id: AddOrder}
    pub bid: PriceLevel,

    pub equibrium_price: Option<EquilibriumPrice>,
    pub trading_status: Option<String>,
}

pub type PriceLevel = BTreeMap<i64, BTreeMap<u64, AddOrder>>;

impl OrderBook {
    /// creates new orderbook with ProductInfo
    pub fn new(r: ProductInfo) -> Self {
        Self {
            product_info: r,
            legs: vec![],
            tick_info: Vec::with_capacity(5),
            orders: HashMap::new(),
            ask: BTreeMap::new(),
            bid: BTreeMap::new(),
            equibrium_price: None,
            trading_status: None,
        }
    }

    /// fetches a single order from OrderBook
    pub fn order(&self, order_id: &u64, side: &Side) -> Option<&AddOrder> {
        let half = match side {
            Side::Buy => &self.ask,
            Side::Sell => &self.bid,
        };
        let key = (*order_id, *side);
        if let Some(price) = self.orders.get(&key) {
            if let Some(order_map) = half.get(price) {
                if let Some(put) = order_map.get(&order_id) {
                    return Some(put);
                }
            }
        };

        None
    }

    /// returns order_book_id
    pub fn order_book_id(&self) -> u64 {
        self.product_info.order_book_id
    }

    // append l message. This message contains information about tick size
    pub fn append_l(&mut self, l: TickSize) {
        self.tick_info.push(l);
    }

    /// handles delete order message.
    /// put ordere returned is the deleted put order
    pub fn delete(&mut self, d: &DeleteOrder) -> AddOrder {
        let id = d.order_id;
        let side = d.side;
        let tree = match side {
            Side::Sell => &mut self.ask,
            Side::Buy => &mut self.bid,
        };

        let func_d = || format!("{:?}", d);
        let price = if let Some(p) = self.orders.remove(&(id, side)) {
            p
        } else {
            unreachable!("{}", func_d())
        };

        let price_level = if let Some(l) = tree.get_mut(&price) {
            l
        } else {
            unreachable!("{}", func_d())
        };

        let a = if let Some(a) = price_level.remove(&id) {
            a
        } else {
            unreachable!("{}", func_d())
        };

        if price_level.len() == 0 {
            tree.remove(&price);
        }

        a
    }

    /// inserts new order onto orderbook
    pub fn put(&mut self, a: AddOrder) {
        let tree = match a.side {
            Side::Sell => &mut self.ask,
            Side::Buy => &mut self.bid,
        };

        if let Some(i) = self.orders.insert((a.order_id, a.side), a.price) {
            let this = tree.get(&i).unwrap();
            unreachable!("\n{this:#?}\n{:#?}\n{i:#?}\n{a:#?}", self.product_info)
        }

        tree.entry(a.price)
            .and_modify(|t| {
                let opts = t.insert(a.order_id, a);
                assert!(opts.is_none());
            })
            .or_insert({
                let mut t = BTreeMap::new();
                t.insert(a.order_id, a);
                t
            });
    }

    /// Handles E message:
    /// Reduces the quantity of an order which is executed against.
    pub fn executed(&mut self, e: &Executed) -> AddOrder {
        let tree = match e.side {
            Side::Sell => &mut self.ask,
            Side::Buy => &mut self.bid,
        };

        let func_e = || format!("{:?}", e);

        let price = if let Some(p) = self.orders.get(&(e.order_id, e.side)) {
            *p
        } else {
            unreachable!("{}", func_e())
        };

        let level = if let Some(l) = tree.get_mut(&price) {
            l
        } else {
            unreachable!("{}", func_e())
        };

        let mut opts = None;
        let check = if let Some(a) = level.get_mut(&e.order_id) {
            a.quantity -= e.executed_quantity;
            debug_assert!(a.quantity >= 0);
            opts.replace(a.clone());
            a.quantity == 0
        } else {
            unreachable!("{}", func_e())
        };

        if check {
            let _ = self.orders.remove(&(e.order_id, e.side));
            let _ = level.remove(&e.order_id);
        }

        if level.len() == 0 {
            tree.remove(&price);
        }

        opts.unwrap()
    }

    /// Handles C message:   
    ///
    /// Reduces the quantity of an order which is executed.  
    ///
    /// returns the AddOrder of which it was affected.   
    ///
    /// The put order is cloned and the same order may remain on the orderbook.
    ///
    pub fn c_executed(&mut self, c: &ExecutionWithPriceInfo) -> AddOrder {
        let tree = match c.side {
            Side::Sell => &mut self.ask,
            Side::Buy => &mut self.bid,
        };

        let func_e = || format!("{:?}", c);

        let price = if let Some(p) = self.orders.get(&(c.order_id, c.side)) {
            *p
        } else {
            unreachable!("{}", func_e())
        };

        let level = if let Some(l) = tree.get_mut(&price) {
            l
        } else {
            unreachable!("{}", func_e())
        };

        let mut opts = None;
        let check = if let Some(a) = level.get_mut(&c.order_id) {
            a.quantity -= c.executed_quantity;
            assert!(a.quantity >= 0);
            opts.replace(a.clone());
            a.quantity == 0
        } else {
            unreachable!("{}", func_e())
        };

        if check {
            let _ = self.orders.remove(&(c.order_id, c.side));
            let _ = level.remove(&c.order_id);
        }

        if level.len() == 0 {
            tree.remove(&price);
        }

        opts.unwrap()
    }

    pub fn set_last_equilibrium_price(&mut self, z: EquilibriumPrice) {
        self.equibrium_price.replace(z);
    }

    pub fn set_trading_status(&mut self, s: &TradingStatusInfo) {
        self.trading_status.replace(s.state_name.clone());
    }
}
