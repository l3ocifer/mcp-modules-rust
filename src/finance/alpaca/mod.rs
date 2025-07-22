use crate::error::{Error, Result};
use crate::lifecycle::LifecycleManager;
use serde::{Deserialize, Serialize};
use serde_json::json;
use chrono::{DateTime, Utc, Duration};
use reqwest::Client;

/// Trading account information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    /// Account ID
    pub id: String,
    /// Account status
    pub status: String,
    /// Account currency
    pub currency: String,
    /// Buying power
    pub buying_power: String,
    /// Cash balance
    pub cash: String,
    /// Portfolio value
    pub portfolio_value: String,
    /// Equity value
    pub equity: String,
    /// Long market value
    pub long_market_value: String,
    /// Short market value
    pub short_market_value: String,
    /// Whether the account is a pattern day trader
    pub pattern_day_trader: bool,
    /// Day trades remaining
    pub daytrade_count: Option<i32>,
}

/// Position in a security
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    /// Symbol
    pub symbol: String,
    /// Quantity of shares
    pub qty: String,
    /// Market value
    pub market_value: String,
    /// Average entry price
    pub avg_entry_price: String,
    /// Current price
    pub current_price: String,
    /// Unrealized profit/loss
    pub unrealized_pl: String,
    /// Unrealized profit/loss percentage
    pub unrealized_plpc: String,
}

/// Stock quote information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quote {
    /// Symbol
    pub symbol: String,
    /// Ask price
    pub ask_price: f64,
    /// Bid price
    pub bid_price: f64,
    /// Ask size
    pub ask_size: i64,
    /// Bid size
    pub bid_size: i64,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Historical price bar
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bar {
    /// Symbol
    pub symbol: String,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Open price
    pub open: f64,
    /// High price
    pub high: f64,
    /// Low price
    pub low: f64,
    /// Close price
    pub close: f64,
    /// Volume
    pub volume: i64,
}

/// Order side enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum OrderSide {
    /// Buy order
    Buy,
    /// Sell order
    Sell,
}

/// Order type enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OrderType {
    /// Market order
    #[serde(rename = "market")]
    Market,
    /// Limit order
    #[serde(rename = "limit")]
    Limit,
    /// Stop order
    #[serde(rename = "stop")]
    Stop,
    /// Stop limit order
    #[serde(rename = "stop_limit")]
    StopLimit,
}

/// Time in force enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TimeInForce {
    /// Day order
    #[serde(rename = "day")]
    Day,
    /// Good till canceled
    #[serde(rename = "gtc")]
    Gtc,
    /// Immediate or cancel
    #[serde(rename = "ioc")]
    Ioc,
    /// Fill or kill
    #[serde(rename = "fok")]
    Fok,
}

/// Order status enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum OrderStatus {
    /// New order
    New,
    /// Partially filled order
    PartiallyFilled,
    /// Filled order
    Filled,
    /// Done for day order
    DoneForDay,
    /// Canceled order
    Canceled,
    /// Expired order
    Expired,
    /// Replaced order
    Replaced,
    /// Pending cancel order
    PendingCancel,
    /// Pending replace order
    PendingReplace,
    /// Accepted order
    Accepted,
    /// Pending new order
    PendingNew,
    /// Accepted for bidding order
    AcceptedForBidding,
    /// Stopped order
    Stopped,
    /// Rejected order
    Rejected,
    /// Suspended order
    Suspended,
    /// Calculated order
    Calculated,
    /// Held order
    Held,
}

/// Order information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    /// Order ID
    pub id: String,
    /// Symbol
    pub symbol: String,
    /// Order type
    pub order_type: OrderType,
    /// Order side
    pub side: OrderSide,
    /// Quantity
    pub qty: String,
    /// Limit price
    pub limit_price: Option<String>,
    /// Stop price
    pub stop_price: Option<String>,
    /// Order status
    pub status: OrderStatus,
    /// Time in force
    pub time_in_force: TimeInForce,
    /// Submitted timestamp
    pub submitted_at: DateTime<Utc>,
    /// Filled timestamp
    pub filled_at: Option<DateTime<Utc>>,
    /// Filled average price
    pub filled_avg_price: Option<String>,
}

/// Market order request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketOrderRequest {
    /// Symbol
    pub symbol: String,
    /// Quantity
    pub qty: f64,
    /// Order side
    pub side: OrderSide,
    /// Time in force
    pub time_in_force: TimeInForce,
}

/// Limit order request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LimitOrderRequest {
    /// Symbol
    pub symbol: String,
    /// Quantity
    pub qty: f64,
    /// Order side
    pub side: OrderSide,
    /// Limit price
    pub limit_price: f64,
    /// Time in force
    pub time_in_force: TimeInForce,
}

/// Order query type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrderQueryType {
    /// Open orders
    #[serde(rename = "open")]
    Open,
    /// Closed orders
    #[serde(rename = "closed")]
    Closed,
    /// All orders
    #[serde(rename = "all")]
    All,
}

/// Client for Alpaca trading API
pub struct AlpacaClient<'a> {
    /// Lifecycle manager
    #[allow(dead_code)]
    lifecycle: &'a LifecycleManager,
    /// HTTP client
    client: Client,
    /// API key
    api_key: String,
    /// API secret
    api_secret: String,
    /// Base URL for API
    base_url: String,
    /// Base URL for data API
    data_base_url: String,
}

impl<'a> AlpacaClient<'a> {
    /// Create a new Alpaca client
    pub fn new(lifecycle: &'a LifecycleManager) -> Self {
        Self {
            lifecycle,
            client: Client::new(),
            api_key: String::new(),
            api_secret: String::new(),
            base_url: "https://paper-api.alpaca.markets".to_string(),
            data_base_url: "https://data.alpaca.markets".to_string(),
        }
    }

    /// Set API key and secret
    pub fn with_credentials(mut self, api_key: impl Into<String>, api_secret: impl Into<String>) -> Self {
        self.api_key = api_key.into();
        self.api_secret = api_secret.into();
        self
    }

    /// Set to use paper trading
    pub fn paper_trading(mut self, paper: bool) -> Self {
        if paper {
            self.base_url = "https://paper-api.alpaca.markets".to_string();
        } else {
            self.base_url = "https://api.alpaca.markets".to_string();
        }
        self
    }

    /// Check if API credentials are available
    pub fn check_credentials(&self) -> Result<()> {
        if self.api_key.is_empty() || self.api_secret.is_empty() {
            return Err(Error::config("Alpaca API credentials not configured"));
        }
        Ok(())
    }

    /// Get account information
    pub async fn get_account(&self) -> Result<Account> {
        self.check_credentials()?;
        
        let url = format!("{}/v2/account", self.base_url);
        let response = self.client
            .get(&url)
            .header("APCA-API-KEY-ID", &self.api_key)
            .header("APCA-API-SECRET-KEY", &self.api_secret)
            .send()
            .await
            .map_err(|e| Error::network(format!("Failed to get account information: {}", e)))?;
            
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await
                .unwrap_or_else(|_| "Unable to read error response".into());
            return Err(Error::network(format!("Alpaca API returned error {}: {}", status, text)));
        }
        
        let account: Account = response.json()
            .await
            .map_err(|e| Error::network(format!("Failed to parse account response: {}", e)))?;
            
        Ok(account)
    }
    
    /// Get positions
    pub async fn get_positions(&self) -> Result<Vec<Position>> {
        self.check_credentials()?;
        
        let url = format!("{}/v2/positions", self.base_url);
        let response = self.client
            .get(&url)
            .header("APCA-API-KEY-ID", &self.api_key)
            .header("APCA-API-SECRET-KEY", &self.api_secret)
            .send()
            .await
            .map_err(|e| Error::network(format!("Failed to get positions: {}", e)))?;
            
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await
                .unwrap_or_else(|_| "Unable to read error response".into());
            return Err(Error::network(format!("Alpaca API returned error {}: {}", status, text)));
        }
        
        let positions: Vec<Position> = response.json()
            .await
            .map_err(|e| Error::network(format!("Failed to parse positions response: {}", e)))?;
            
        Ok(positions)
    }
    
    /// Get latest quote for a stock
    pub async fn get_stock_quote(&self, symbol: &str) -> Result<Quote> {
        self.check_credentials()?;
        
        let url = format!("{}/v2/stocks/{}/quotes/latest", self.data_base_url, symbol);
        let response = self.client
            .get(&url)
            .header("APCA-API-KEY-ID", &self.api_key)
            .header("APCA-API-SECRET-KEY", &self.api_secret)
            .send()
            .await
            .map_err(|e| Error::network(format!("Failed to get stock quote: {}", e)))?;
            
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await
                .unwrap_or_else(|_| "Unable to read error response".into());
            return Err(Error::network(format!("Alpaca API returned error {}: {}", status, text)));
        }
        
        #[derive(Deserialize)]
        struct QuoteResponse {
            quote: Quote,
        }
        
        let response_data: QuoteResponse = response.json()
            .await
            .map_err(|e| Error::network(format!("Failed to parse quote response: {}", e)))?;
            
        Ok(response_data.quote)
    }
    
    /// Get historical bars for a stock
    pub async fn get_stock_bars(&self, symbol: &str, days: i64) -> Result<Vec<Bar>> {
        self.check_credentials()?;
        
        let start_time = Utc::now() - Duration::days(days);
        let start_str = start_time.format("%Y-%m-%dT%H:%M:%SZ").to_string();
        
        let url = format!(
            "{}/v2/stocks/{}/bars?timeframe=1D&start={}&adjustment=raw", 
            self.data_base_url, 
            symbol,
            start_str
        );
        
        let response = self.client
            .get(&url)
            .header("APCA-API-KEY-ID", &self.api_key)
            .header("APCA-API-SECRET-KEY", &self.api_secret)
            .send()
            .await
            .map_err(|e| Error::network(format!("Failed to get stock bars: {}", e)))?;
            
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await
                .unwrap_or_else(|_| "Unable to read error response".into());
            return Err(Error::network(format!("Alpaca API returned error {}: {}", status, text)));
        }
        
        #[derive(Deserialize)]
        struct BarResponse {
            bars: Vec<Bar>,
        }
        
        let response_data: BarResponse = response.json()
            .await
            .map_err(|e| Error::network(format!("Failed to parse bars response: {}", e)))?;
            
        Ok(response_data.bars)
    }
    
    /// Get orders
    pub async fn get_orders(&self, status: OrderQueryType, limit: usize) -> Result<Vec<Order>> {
        self.check_credentials()?;
        
        let status_str = match status {
            OrderQueryType::Open => "open",
            OrderQueryType::Closed => "closed",
            OrderQueryType::All => "all",
        };
        
        let url = format!(
            "{}/v2/orders?status={}&limit={}", 
            self.base_url, 
            status_str,
            limit
        );
        
        let response = self.client
            .get(&url)
            .header("APCA-API-KEY-ID", &self.api_key)
            .header("APCA-API-SECRET-KEY", &self.api_secret)
            .send()
            .await
            .map_err(|e| Error::network(format!("Failed to get orders: {}", e)))?;
            
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await
                .unwrap_or_else(|_| "Unable to read error response".into());
            return Err(Error::network(format!("Alpaca API returned error {}: {}", status, text)));
        }
        
        let orders: Vec<Order> = response.json()
            .await
            .map_err(|e| Error::network(format!("Failed to parse orders response: {}", e)))?;
            
        Ok(orders)
    }
    
    /// Place a market order
    pub async fn place_market_order(&self, request: MarketOrderRequest) -> Result<Order> {
        self.check_credentials()?;
        
        let url = format!("{}/v2/orders", self.base_url);
        
        #[derive(Serialize)]
        struct OrderRequest {
            symbol: String,
            qty: String,
            side: OrderSide,
            type_: OrderType,
            time_in_force: TimeInForce,
        }
        
        let order_request = OrderRequest {
            symbol: request.symbol,
            qty: request.qty.to_string(),
            side: request.side,
            type_: OrderType::Market,
            time_in_force: request.time_in_force,
        };
        
        let response = self.client
            .post(&url)
            .header("APCA-API-KEY-ID", &self.api_key)
            .header("APCA-API-SECRET-KEY", &self.api_secret)
            .json(&json!({
                "symbol": order_request.symbol,
                "qty": order_request.qty,
                "side": order_request.side,
                "type": "market",
                "time_in_force": order_request.time_in_force,
            }))
            .send()
            .await
            .map_err(|e| Error::network(format!("Failed to place market order: {}", e)))?;
            
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await
                .unwrap_or_else(|_| "Unable to read error response".into());
            return Err(Error::network(format!("Alpaca API returned error {}: {}", status, text)));
        }
        
        let order: Order = response.json()
            .await
            .map_err(|e| Error::network(format!("Failed to parse order response: {}", e)))?;
            
        Ok(order)
    }
    
    /// Place a limit order
    pub async fn place_limit_order(&self, request: LimitOrderRequest) -> Result<Order> {
        self.check_credentials()?;
        
        let url = format!("{}/v2/orders", self.base_url);
        
        let response = self.client
            .post(&url)
            .header("APCA-API-KEY-ID", &self.api_key)
            .header("APCA-API-SECRET-KEY", &self.api_secret)
            .json(&json!({
                "symbol": request.symbol,
                "qty": request.qty.to_string(),
                "side": request.side,
                "type": "limit",
                "time_in_force": request.time_in_force,
                "limit_price": request.limit_price.to_string(),
            }))
            .send()
            .await
            .map_err(|e| Error::network(format!("Failed to place limit order: {}", e)))?;
            
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await
                .unwrap_or_else(|_| "Unable to read error response".into());
            return Err(Error::network(format!("Alpaca API returned error {}: {}", status, text)));
        }
        
        let order: Order = response.json()
            .await
            .map_err(|e| Error::network(format!("Failed to parse order response: {}", e)))?;
            
        Ok(order)
    }
    
    /// Cancel an order
    pub async fn cancel_order(&self, order_id: &str) -> Result<()> {
        self.check_credentials()?;
        
        let url = format!("{}/v2/orders/{}", self.base_url, order_id);
        
        let response = self.client
            .delete(&url)
            .header("APCA-API-KEY-ID", &self.api_key)
            .header("APCA-API-SECRET-KEY", &self.api_secret)
            .send()
            .await
            .map_err(|e| Error::network(format!("Failed to cancel order: {}", e)))?;
            
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await
                .unwrap_or_else(|_| "Unable to read error response".into());
            return Err(Error::network(format!("Alpaca API returned error {}: {}", status, text)));
        }
        
        Ok(())
    }

    /// Get registered tools
    pub fn get_tools(&self) -> Vec<(String, String, serde_json::Value)> {
        vec![
            (
                "get_account_info".to_string(),
                "Get the current account information including balances and status".to_string(),
                serde_json::json!({
                    "type": "object",
                    "properties": {}
                }),
            ),
            (
                "get_positions".to_string(),
                "Get all current positions in the portfolio".to_string(),
                serde_json::json!({
                    "type": "object",
                    "properties": {}
                }),
            ),
            (
                "get_stock_quote".to_string(),
                "Get the latest quote for a stock".to_string(),
                serde_json::json!({
                    "type": "object",
                    "required": ["symbol"],
                    "properties": {
                        "symbol": {
                            "type": "string",
                            "description": "Stock ticker symbol (e.g., AAPL, MSFT)"
                        }
                    }
                }),
            ),
            (
                "get_stock_bars".to_string(),
                "Get historical price bars for a stock".to_string(),
                serde_json::json!({
                    "type": "object",
                    "required": ["symbol"],
                    "properties": {
                        "symbol": {
                            "type": "string",
                            "description": "Stock ticker symbol (e.g., AAPL, MSFT)"
                        },
                        "days": {
                            "type": "integer",
                            "description": "Number of trading days to look back (default: 5)"
                        }
                    }
                }),
            ),
            (
                "get_orders".to_string(),
                "Get orders with the specified status".to_string(),
                serde_json::json!({
                    "type": "object",
                    "properties": {
                        "status": {
                            "type": "string",
                            "enum": ["open", "closed", "all"],
                            "description": "Order status to filter by (open, closed, all)"
                        },
                        "limit": {
                            "type": "integer",
                            "description": "Maximum number of orders to return (default: 10)"
                        }
                    }
                }),
            ),
            (
                "place_market_order".to_string(),
                "Place a market order".to_string(),
                serde_json::json!({
                    "type": "object",
                    "required": ["symbol", "side", "quantity"],
                    "properties": {
                        "symbol": {
                            "type": "string",
                            "description": "Stock ticker symbol (e.g., AAPL, MSFT)"
                        },
                        "side": {
                            "type": "string",
                            "enum": ["buy", "sell"],
                            "description": "Order side (buy or sell)"
                        },
                        "quantity": {
                            "type": "number",
                            "description": "Number of shares to buy or sell"
                        }
                    }
                }),
            ),
            (
                "place_limit_order".to_string(),
                "Place a limit order".to_string(),
                serde_json::json!({
                    "type": "object",
                    "required": ["symbol", "side", "quantity", "limit_price"],
                    "properties": {
                        "symbol": {
                            "type": "string",
                            "description": "Stock ticker symbol (e.g., AAPL, MSFT)"
                        },
                        "side": {
                            "type": "string",
                            "enum": ["buy", "sell"],
                            "description": "Order side (buy or sell)"
                        },
                        "quantity": {
                            "type": "number",
                            "description": "Number of shares to buy or sell"
                        },
                        "limit_price": {
                            "type": "number",
                            "description": "Limit price for the order"
                        }
                    }
                }),
            ),
            (
                "cancel_order".to_string(),
                "Cancel an existing order".to_string(),
                serde_json::json!({
                    "type": "object",
                    "required": ["order_id"],
                    "properties": {
                        "order_id": {
                            "type": "string",
                            "description": "Order ID to cancel"
                        }
                    }
                }),
            ),
        ]
    }
} 