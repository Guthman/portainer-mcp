//! MCP server for managing Docker Compose stacks on a Portainer instance.
//!
//! Communicates over stdio using JSON-RPC and exposes typed tools for common
//! stack operations plus a generic fallback for full Portainer API access.
//!
//! # Configuration
//!
//! Set these environment variables before starting the server:
//!
//! | Variable | Required | Default | Description |
//! |---|---|---|---|
//! | `PORTAINER_API_KEY` | Yes | — | Portainer API key |
//! | `PORTAINER_URL` | No | `http://localhost:9000` | Portainer instance URL |
//! | `PORTAINER_INSECURE` | No | `false` | Accept self-signed TLS certs |

pub mod client;
pub mod models;
pub mod redact;
pub mod server;
