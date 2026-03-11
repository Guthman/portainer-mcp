# portainer-mcp

An [MCP](https://modelcontextprotocol.io/) server for managing Docker Compose stacks on a [Portainer](https://www.portainer.io/) instance. Exposes 10 typed tools for common stack operations plus a generic fallback for full API access.

## Features

- List, inspect, create, update, and delete compose stacks
- Start, stop, and git-redeploy stacks
- List environments/endpoints
- Generic request tool for any Portainer API endpoint
- API key authentication (no login flow needed)
- Optional self-signed certificate support

## Installation

### From source

```sh
git clone https://github.com/Guthman/portainer-mcp.git
cd portainer-mcp
cargo build --release
```

The binary will be at `target/release/portainer-mcp` (or `portainer-mcp.exe` on Windows).

### Pre-built binaries

Check [Releases](https://github.com/Guthman/portainer-mcp/releases) for pre-built binaries.

## Configuration

Set these environment variables:

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `PORTAINER_API_KEY` | Yes | — | Portainer API key ([how to create one](https://docs.portainer.io/api/access)) |
| `PORTAINER_URL` | No | `http://localhost:9000` | Portainer instance URL |
| `PORTAINER_INSECURE` | No | `false` | Set to `true` to accept self-signed TLS certificates |

## Usage

### With Claude Desktop

Add to your Claude Desktop MCP config (`claude_desktop_config.json`):

```json
{
  "mcpServers": {
    "portainer": {
      "command": "path/to/portainer-mcp",
      "env": {
        "PORTAINER_URL": "https://your-portainer:9443",
        "PORTAINER_API_KEY": "your-api-key",
        "PORTAINER_INSECURE": "true"
      }
    }
  }
}
```

### With Claude Code

Add to your `.mcp.json`:

```json
{
  "mcpServers": {
    "portainer": {
      "command": "path/to/portainer-mcp",
      "env": {
        "PORTAINER_URL": "https://your-portainer:9443",
        "PORTAINER_API_KEY": "your-api-key",
        "PORTAINER_INSECURE": "true"
      }
    }
  }
}
```

## Tools

| Tool | Description |
|------|-------------|
| `list_endpoints` | List environments/endpoints — call this first to get endpoint IDs |
| `list_stacks` | List all stacks, optionally filtered |
| `get_stack` | Get a single stack by ID |
| `get_stack_file` | Get the compose file content of a stack |
| `create_stack` | Create a new compose stack from file content |
| `update_stack` | Update a stack's compose file, env vars, or settings |
| `delete_stack` | Delete a stack |
| `start_stack` | Start a stopped stack |
| `stop_stack` | Stop a running stack |
| `redeploy_git_stack` | Redeploy a git-based stack (pull latest and redeploy) |
| `portainer_request` | Generic API request for any Portainer endpoint |

### Typical workflow

1. `list_endpoints` — get the `endpoint_id` for your environment
2. `list_stacks` — see your stacks
3. `get_stack_file` — inspect a stack's compose file
4. `update_stack` / `redeploy_git_stack` — make changes

## License

MIT