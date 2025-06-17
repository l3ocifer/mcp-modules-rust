/// Alpaca trading module for stock market trading
pub mod alpaca;

// Re-export key types
pub use alpaca::{AlpacaClient, Account, Position, Quote, Bar, OrderSide, OrderType, TimeInForce, Order}; 