# portainer-mcp

An [MCP](https://modelcontextprotocol.io/) server for managing Docker Compose stacks on a [Portainer](https://www.portainer.io/) instance. Built against the Portainer 2.39.0 API spec. Exposes 10 typed tools for common stack operations plus a generic fallback for full API access.

> For broader Portainer coverage beyond stack management, see the official [portainer/portainer-mcp](https://github.com/portainer/portainer-mcp).

## Why this server?

- **Minimal and focused** — purpose-built for stack management
- **Single static binary** — no runtime, no `node_modules`, trivial to install from [pre-built releases](https://github.com/Guthman/portainer-mcp/releases)
- **Low memory footprint** — a few MB vs Node's ~30-50MB baseline; ideal for a process that sits idle between tool calls
- **Instant startup** — no module loading delay; relevant since MCP hosts may spawn/kill the process per session
- **MCP-native** — includes tool annotations, prompts, and resources

## Features

- List, inspect, create, update, and delete compose stacks
- Start, stop, and git-redeploy stacks
- List environments/endpoints
- Generic request tool for any Portainer API endpoint — if a dedicated tool doesn't cover your use case, the fallback `portainer_request` tool gives full access to the entire Portainer API
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

## Roadmap

- Automatic tool updates when new Portainer API versions are released

## License

MIT