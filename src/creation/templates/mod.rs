use crate::creation::ServerLanguage;

/// Get server template code for a specific language
pub fn get_template_code(language: &ServerLanguage) -> &'static str {
    match language {
        ServerLanguage::TypeScript => TYPESCRIPT_TEMPLATE,
        ServerLanguage::JavaScript => JAVASCRIPT_TEMPLATE,
        ServerLanguage::Python => PYTHON_TEMPLATE,
        ServerLanguage::Rust => RUST_TEMPLATE,
    }
}

/// TypeScript MCP server template
pub const TYPESCRIPT_TEMPLATE: &str = r#"
import { Server } from "@modelcontextprotocol/sdk/server/index.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import { 
  CallToolRequestSchema, 
  ListToolsRequestSchema 
} from "@modelcontextprotocol/sdk/types.js";

const server = new Server({
  name: "dynamic-test-server",
  version: "1.0.0"
}, {
  capabilities: {
    tools: {}
  }
});

// Server implementation
server.setRequestHandler(ListToolsRequestSchema, async () => {
  return {
    tools: [{
      name: "echo",
      description: "Echo back a message",
      inputSchema: {
        type: "object",
        properties: {
          message: { type: "string" }
        },
        required: ["message"]
      }
    }]
  };
});

server.setRequestHandler(CallToolRequestSchema, async (request) => {
  if (request.params.name === "echo") {
    return {
      content: [
        {
          type: "text",
          text: `Echo: ${request.params.arguments.message}`
        }
      ]
    };
  }
  throw new Error("Tool not found");
});

// Server startup
const transport = new StdioServerTransport();
server.connect(transport);
"#;

/// JavaScript MCP server template
pub const JAVASCRIPT_TEMPLATE: &str = r#"
const { Server } = require("@modelcontextprotocol/sdk/server/index.js");
const { StdioServerTransport } = require("@modelcontextprotocol/sdk/server/stdio.js");
const { CallToolRequestSchema, ListToolsRequestSchema } = require("@modelcontextprotocol/sdk/types.js");

const server = new Server({
  name: "dynamic-test-server",
  version: "1.0.0"
}, {
  capabilities: {
    tools: {}
  }
});

// Server implementation
server.setRequestHandler(ListToolsRequestSchema, async () => {
  return {
    tools: [{
      name: "echo",
      description: "Echo back a message",
      inputSchema: {
        type: "object",
        properties: {
          message: { type: "string" }
        },
        required: ["message"]
      }
    }]
  };
});

server.setRequestHandler(CallToolRequestSchema, async (request) => {
  if (request.params.name === "echo") {
    return {
      content: [
        {
          type: "text",
          text: `Echo: ${request.params.arguments.message}`
        }
      ]
    };
  }
  throw new Error("Tool not found");
});

// Server startup
const transport = new StdioServerTransport();
server.connect(transport);
"#;

/// Python MCP server template
pub const PYTHON_TEMPLATE: &str = r#"
#!/usr/bin/env python3
from mcp.client import ClientRequestSchema
from mcp.server.fastmcp import FastMCP
from mcp.server.schema import CallToolRequestSchema, ListToolsRequestSchema
from mcp.shared.types import ToolDefinition
from pydantic import BaseModel

# Define the server
mcp = FastMCP(name="dynamic-test-server")

# Define tool input schema
class EchoInput(BaseModel):
    message: str

# Define tools
@mcp.tool()
async def echo(input_data: EchoInput):
    return [
        {
            "type": "text",
            "text": f"Echo: {input_data.message}"
        }
    ]

# Start the server
if __name__ == "__main__":
    mcp.run()
"#;

/// Rust MCP server template
pub const RUST_TEMPLATE: &str = r#"
use mcp::{server::Server, tools::Tool, schemas::CallToolRequest, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Deserialize)]
struct EchoInput {
    message: String,
}

#[derive(Debug, Serialize)]
struct EchoOutput {
    text: String,
}

struct EchoTool;

impl Tool for EchoTool {
    fn name(&self) -> &'static str {
        "echo"
    }

    fn description(&self) -> &'static str {
        "Echo back a message"
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "message": {"type": "string"}
            },
            "required": ["message"]
        })
    }

    async fn execute(&self, request: CallToolRequest) -> Result<Vec<Value>> {
        let input: EchoInput = serde_json::from_value(request.arguments)?;
        Ok(vec![json!({
            "type": "text",
            "text": format!("Echo: {}", input.message)
        })])
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut server = Server::new("dynamic-test-server");
    server.add_tool(Box::new(EchoTool));
    server.run().await
}
"#;
