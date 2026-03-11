# portainer-stacks

An [MCP](https://modelcontextprotocol.io/) server for managing Docker Compose stacks on a [Portainer](https://www.portainer.io/) instance. Built against the Portainer 2.39.0 API spec. Exposes 10 typed tools for common stack operations plus a generic fallback for full API access.

> For broader Portainer coverage beyond stack management, see the official [portainer/portainer-mcp](https://github.com/portainer/portainer-mcp).

## Why this server?

- **Minimal and focused** — purpose-built for stack management
- **Windows, Linux & macOS (arm64 and x86_64)**
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

The binary will be at `target/release/portainer-stacks` (or `portainer-stacks.exe` on Windows).

### Pre-built binaries

Check [Releases](https://github.com/Guthman/portainer-mcp/releases) for pre-built binaries.

### Verifying what you run

When you configure any MCP server, you're telling your AI assistant to run a binary on your machine with whatever permissions you give it. If that binary gets swapped out — a bad download, a compromised update, or something modifying it on disk — the assistant will run the replacement without question. Most MCP configurations don't have a built-in way to pin to a specific binary hash, so it's worth verifying the file yourself.

This applies to all MCP servers, not just this one.

Release binaries for this project include [GitHub Artifact Attestations](https://docs.github.com/en/actions/security-for-github-actions/using-artifact-attestations/using-artifact-attestations-to-establish-provenance-for-builds) — signed provenance records that confirm a binary was built by this repo's CI from a specific commit. To verify a downloaded artifact:

```sh
gh attestation verify portainer-stacks-v*.tar.gz --owner Guthman
```

If verification passes, the binary is untampered and matches what the source code produced. It doesn't guarantee the code itself is safe — you should still review what you run — but it closes the gap between "code in the repo" and "binary on your machine."

## Configuration

Set these environment variables:

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `PORTAINER_API_KEY` | Yes | — | Portainer API key ([how to create one](https://docs.portainer.io/api/access)) |
| `PORTAINER_URL` | No | `http://localhost:9000` | Portainer instance URL |
| `PORTAINER_INSECURE` | No | `false` | Set to `true` to accept self-signed TLS certificates |
| `PORTAINER_ENV_DISPLAY` | No | `masked` | How env var values appear in responses: `masked` (all hidden), `filtered` (sensitive redacted), `full` (all visible) |
| `PORTAINER_SENSITIVE_NAMES` | No | — | Comma-separated env var names to **add** as sensitive in `filtered` mode |
| `PORTAINER_VISIBLE_NAMES` | No | — | Comma-separated env var names to **force** visible in `filtered` mode |

### Environment variable display modes

Stack objects returned by the Portainer API include environment variables that often contain secrets (passwords, API keys, tokens, connection strings). By default, all values are masked before they reach the LLM.

| Mode | Behavior |
|------|----------|
| `masked` | All env var values → `[MASKED]` |
| `filtered` | Names matching sensitive patterns → `[REDACTED]`, others shown |
| `full` | All values in cleartext (use with caution) |

In `filtered` mode, names matching built-in patterns (PASSWORD, SECRET, TOKEN, etc.) and any names listed in `PORTAINER_SENSITIVE_NAMES` are redacted. Names in `PORTAINER_VISIBLE_NAMES` override both, so you can exempt specific variables. Priority: explicit visible > explicit sensitive > built-in pattern.

Use the `configure-env-display` prompt to scan your stacks and get personalized guidance.

## Usage

### With Claude Desktop

Add to your Claude Desktop MCP config (`claude_desktop_config.json`):

```json
{
  "mcpServers": {
    "portainer": {
      "command": "path/to/portainer-stacks",
      "env": {
        "PORTAINER_URL": "https://your-portainer:9443",
        "PORTAINER_API_KEY": "your-api-key",
        "PORTAINER_INSECURE": "true",
        "PORTAINER_ENV_DISPLAY": "masked"
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
      "command": "path/to/portainer-stacks",
      "env": {
        "PORTAINER_URL": "https://your-portainer:9443",
        "PORTAINER_API_KEY": "your-api-key",
        "PORTAINER_INSECURE": "true",
        "PORTAINER_ENV_DISPLAY": "masked"
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
- Support for more auth methods or credential handling? Better prompts? Let me know!

## License

MIT