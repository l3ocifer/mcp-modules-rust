#!/usr/bin/env node

/**
 * MCP Stdio Bridge for Remote Cursor Integration
 * This script bridges stdio communication from Cursor to the remote MCP server via Cloudflare tunnel
 */

const https = require('https');
const readline = require('readline');

// Use environment variable or default to remote URL
const MCP_SERVER_URL = process.env.MCP_SERVER_URL || 'https://mcp.leopaska.xyz';

// Create readline interface for stdio
const rl = readline.createInterface({
  input: process.stdin,
  output: process.stdout,
  terminal: false
});

// Function to send HTTPS request to our remote MCP server
function sendToMCPServer(data) {
  return new Promise((resolve, reject) => {
    const postData = JSON.stringify(data);
    
    const url = new URL(MCP_SERVER_URL);
    const options = {
      hostname: url.hostname,
      port: url.port || 443,
      path: url.pathname || '/',
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Content-Length': Buffer.byteLength(postData),
        'User-Agent': 'MCP-Stdio-Bridge/1.0'
      }
    };

    const req = https.request(options, (res) => {
      let responseData = '';
      
      res.on('data', (chunk) => {
        responseData += chunk;
      });
      
      res.on('end', () => {
        try {
          const response = JSON.parse(responseData);
          resolve(response);
        } catch (error) {
          reject(new Error(`Failed to parse response: ${error.message}`));
        }
      });
    });

    req.on('error', (error) => {
      reject(new Error(`HTTPS request failed: ${error.message}`));
    });

    req.write(postData);
    req.end();
  });
}

// Handle incoming lines from stdin (Cursor)
rl.on('line', async (line) => {
  try {
    // Parse the JSON-RPC request from Cursor
    const request = JSON.parse(line.trim());
    
    // Forward to our remote MCP server
    const response = await sendToMCPServer(request);
    
    // Send response back to Cursor via stdout
    console.log(JSON.stringify(response));
  } catch (error) {
    // Send error response back to Cursor
    const errorResponse = {
      jsonrpc: "2.0",
      id: null,
      error: {
        code: -32603,
        message: `Bridge error: ${error.message}`
      }
    };
    console.log(JSON.stringify(errorResponse));
  }
});

// Handle process termination
process.on('SIGINT', () => {
  process.exit(0);
});

process.on('SIGTERM', () => {
  process.exit(0);
});

// Send initial message to stderr for debugging (won't interfere with JSON-RPC)
console.error('MCP remote stdio bridge started, connecting to', MCP_SERVER_URL);
