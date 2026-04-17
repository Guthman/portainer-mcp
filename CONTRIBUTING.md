# Contributing

## Development

```sh
cargo build                  # Debug build
cargo check                  # Type-check without building
cargo fmt                    # Auto-format
cargo fmt --all -- --check   # Check formatting
cargo clippy -- -D warnings  # Lint (CI treats warnings as errors)
cargo test --lib             # Unit tests (no external services needed)
```

The integration tests in `tests/` require a live Portainer instance (see below).

### Integration tests

Set `PORTAINER_URL` and `PORTAINER_API_KEY` before running:

```sh
cargo test
```

The test harness in `tests/common/mod.rs` can also spin up a Portainer container via [testcontainers](https://crates.io/crates/testcontainers) — this requires Docker to be available.

## Releasing

This project uses [semantic versioning](https://semver.org/). Releases are automated via GitHub Actions — pushing a `v*` tag triggers the release workflow, which builds cross-platform binaries and creates a GitHub release with artifact attestations.

### Steps

1. **Decide the version bump:**
   - **Patch** (`0.4.0` -> `0.4.1`) — bug fixes, dependency patches with no behavior change
   - **Minor** (`0.4.0` -> `0.5.0`) — new features, non-breaking dependency upgrades, new tools
   - **Major** (`0.4.0` -> `1.0.0`) — breaking changes to the MCP tool interface or configuration

2. **Update the version in `Cargo.toml`:**
   ```sh
   # Edit Cargo.toml: version = "X.Y.Z"
   cargo check  # Regenerates Cargo.lock
   ```

3. **Commit the version bump:**
   ```sh
   git add Cargo.toml Cargo.lock
   git commit -m "chore: bump version to X.Y.Z"
   ```

4. **Tag and push:**
   ```sh
   git tag vX.Y.Z
   git push && git push origin vX.Y.Z
   ```

5. **Verify the release:**
   - Check the [Actions tab](https://github.com/Guthman/portainer-mcp/actions) for the release workflow
   - Once complete, binaries appear on the [Releases page](https://github.com/Guthman/portainer-mcp/releases)
   - Verify a downloaded artifact: `gh attestation verify portainer-stacks-v*.tar.gz --owner Guthman`

### Release targets

The release workflow builds for:
- `x86_64-unknown-linux-musl` (Linux, static binary)
- `x86_64-pc-windows-msvc` (Windows)
- `x86_64-apple-darwin` (macOS Intel)
- `aarch64-apple-darwin` (macOS Apple Silicon)
