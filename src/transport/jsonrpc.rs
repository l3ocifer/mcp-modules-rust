use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fmt;
use uuid::Uuid;

use crate::error::{Error, Result};
use crate::Transport;

/// JSON-RPC request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    /// JSON-RPC version
    pub jsonrpc: String,
    /// Request ID
    pub id: String,
    /// Method to call
    pub method: String,
    /// Parameters
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

/// JSON-RPC response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    /// JSON-RPC version
    pub jsonrpc: String,
    /// Response ID matching request
    pub id: String,
    /// Result value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    /// Error information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

/// JSON-RPC notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcNotification {
    /// JSON-RPC version
    pub jsonrpc: String,
    /// Method being called
    pub method: String,
    /// Parameters
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

/// JSON-RPC error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    /// Error code
    pub code: i64,
    /// Error message
    pub message: String,
    /// Additional error data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl fmt::Display for JsonRpcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "JSON-RPC error {}: {}", self.code, self.message)
    }
}

/// JSON-RPC message (request, response, or notification)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JsonRpcMessage {
    /// Request message
    Request(JsonRpcRequest),
    /// Response message
    Response(JsonRpcResponse),
    /// Notification message
    Notification(JsonRpcNotification),
    /// Batch of messages
    Batch(Vec<JsonRpcMessage>),
}

impl JsonRpcMessage {
    /// Create a new request message
    pub fn request(method: &str, params: Option<Value>) -> Self {
        JsonRpcMessage::Request(JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Uuid::new_v4().to_string(),
            method: method.to_string(),
            params,
        })
    }

    /// Create a new response message
    pub fn response(id: &str, result: Value) -> Self {
        JsonRpcMessage::Response(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: id.to_string(),
            result: Some(result),
            error: None,
        })
    }

    /// Create a new error response message
    pub fn error(id: &str, code: i64, message: &str) -> Self {
        JsonRpcMessage::Response(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: id.to_string(),
            result: None,
            error: Some(JsonRpcError {
                code,
                message: message.to_string(),
                data: None,
            }),
        })
    }

    /// Create a new notification message
    pub fn notification(method: &str, params: Option<Value>) -> Self {
        JsonRpcMessage::Notification(JsonRpcNotification {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params,
        })
    }

    /// Create a new batch message
    pub fn batch(messages: Vec<JsonRpcMessage>) -> Self {
        JsonRpcMessage::Batch(messages)
    }

    /// Get the ID of the message (if it has one)
    pub fn id(&self) -> Option<String> {
        match self {
            JsonRpcMessage::Request(req) => Some(req.id.clone()),
            JsonRpcMessage::Response(res) => Some(res.id.clone()),
            _ => None,
        }
    }

    /// Get the method of the message (if it has one)
    pub fn method(&self) -> Option<String> {
        match self {
            JsonRpcMessage::Request(req) => Some(req.method.clone()),
            JsonRpcMessage::Notification(notif) => Some(notif.method.clone()),
            _ => None,
        }
    }

    /// Get the parameters of the message (if it has them)
    pub fn params(&self) -> Option<Value> {
        match self {
            JsonRpcMessage::Request(req) => req.params.clone(),
            JsonRpcMessage::Notification(notif) => notif.params.clone(),
            _ => None,
        }
    }

    /// Get the result of the message (if it has one)
    pub fn result(&self) -> Option<Value> {
        match self {
            JsonRpcMessage::Response(res) => res.result.clone(),
            _ => None,
        }
    }

    /// Get the error of the message (if it has one)
    pub fn get_error(&self) -> Option<JsonRpcError> {
        match self {
            JsonRpcMessage::Response(res) => res.error.clone(),
            _ => None,
        }
    }

    /// Convert to a Message
    pub fn to_message(&self) -> Message {
        match self {
            JsonRpcMessage::Request(req) => Message::Request {
                id: Some(json!(req.id.clone())),
                method: req.method.clone(),
                params: req.params.clone(),
            },
            JsonRpcMessage::Response(res) => Message::Response {
                id: Some(json!(res.id.clone())),
                result: res.result.clone(),
                error: res.error.as_ref().map(|e| json!(e)),
            },
            JsonRpcMessage::Notification(notif) => Message::Notification {
                method: notif.method.clone(),
                params: notif.params.clone(),
            },
            JsonRpcMessage::Batch(messages) => {
                let batch_messages = messages.iter().map(|m| m.to_message()).collect();
                Message::Batch(batch_messages)
            }
        }
    }

    /// Convert from a Message
    pub fn from_message(message: &Message) -> Result<Self> {
        match message {
            Message::Request { id, method, params } => {
                Ok(JsonRpcMessage::Request(JsonRpcRequest {
                    jsonrpc: "2.0".to_string(),
                    id: id
                        .as_ref()
                        .and_then(|i| i.as_str().map(|s| s.to_string()))
                        .unwrap_or_else(|| Uuid::new_v4().to_string()),
                    method: method.clone(),
                    params: params.clone(),
                }))
            }
            Message::Response { id, result, error } => {
                Ok(JsonRpcMessage::Response(JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: id
                        .as_ref()
                        .and_then(|i| i.as_str().map(|s| s.to_string()))
                        .unwrap_or_else(|| Uuid::new_v4().to_string()),
                    result: result.clone(),
                    error: error.as_ref().map(|e| {
                        let code = e.get("code").and_then(|c| c.as_i64()).unwrap_or(-32000);
                        let message = e
                            .get("message")
                            .and_then(|m| m.as_str())
                            .unwrap_or("Unknown error")
                            .to_string();
                        let data = e.get("data").cloned();

                        JsonRpcError {
                            code,
                            message,
                            data,
                        }
                    }),
                }))
            }
            Message::Notification { method, params } => {
                Ok(JsonRpcMessage::Notification(JsonRpcNotification {
                    jsonrpc: "2.0".to_string(),
                    method: method.clone(),
                    params: params.clone(),
                }))
            }
            Message::Batch(messages) => {
                let batch_messages = messages
                    .iter()
                    .map(JsonRpcMessage::from_message)
                    .collect::<Result<Vec<_>>>()?;

                Ok(JsonRpcMessage::Batch(batch_messages))
            }
        }
    }
}

/// JSON-RPC message
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Message {
    /// Request message
    Request {
        /// Request ID
        id: Option<Value>,
        /// Method to call
        method: String,
        /// Parameters
        #[serde(skip_serializing_if = "Option::is_none")]
        params: Option<Value>,
    },
    /// Response message
    Response {
        /// Response ID
        id: Option<Value>,
        /// Result value
        #[serde(skip_serializing_if = "Option::is_none")]
        result: Option<Value>,
        /// Error information
        #[serde(skip_serializing_if = "Option::is_none")]
        error: Option<Value>,
    },
    /// Notification message
    Notification {
        /// Method being called
        method: String,
        /// Parameters
        #[serde(skip_serializing_if = "Option::is_none")]
        params: Option<Value>,
    },
    /// Batch of messages
    Batch(Vec<Message>),
}

impl Message {
    /// Get the error from the message
    pub fn get_error(&self) -> Option<JsonRpcError> {
        match self {
            Message::Request {
                id: _,
                method: _,
                params: _,
            } => None,
            Message::Response {
                id: _,
                result: _,
                error,
            } => error.as_ref().and_then(|e| {
                e.get("code")
                    .and_then(|c| c.as_i64())
                    .map(|code| JsonRpcError {
                        code,
                        message: e
                            .get("message")
                            .and_then(|m| m.as_str())
                            .unwrap_or("Unknown error")
                            .to_string(),
                        data: e.get("data").cloned(),
                    })
            }),
            Message::Notification {
                method: _,
                params: _,
            } => None,
            Message::Batch(messages) => messages.iter().find_map(|m| m.get_error()),
        }
    }
}

/// Create a JSON-RPC request
pub fn create_request(id: &str, method: &str, params: Option<Value>) -> Message {
    Message::Request {
        id: Some(json!(id)),
        method: method.to_string(),
        params,
    }
}

/// Create a JSON-RPC notification (no ID)
pub fn create_notification(method: &str, params: Option<Value>) -> Message {
    Message::Notification {
        method: method.to_string(),
        params,
    }
}

/// Create a JSON-RPC success response
pub fn create_response(id: &str, result: Value) -> Message {
    Message::Response {
        id: Some(json!(id)),
        result: Some(result),
        error: None,
    }
}

/// Create a JSON-RPC error response
pub fn create_error_response(id: &str, code: i64, message: &str) -> Message {
    Message::Response {
        id: Some(json!(id)),
        result: None,
        error: Some(json!({
            "code": code,
            "message": message,
        })),
    }
}

/// Implementation of `From<Message>` for Value
impl From<Message> for Value {
    fn from(message: Message) -> Self {
        serde_json::to_value(message).unwrap_or(Value::Null)
    }
}

/// Perform a request using a transport
pub async fn perform_request<T: Transport>(
    transport: &mut T,
    method: &str,
    params: Option<Value>,
) -> Result<Value> {
    transport
        .request(method, params)
        .await
        .map_err(|e| Error::Transport(crate::error::TransportError::from(e)))
}
