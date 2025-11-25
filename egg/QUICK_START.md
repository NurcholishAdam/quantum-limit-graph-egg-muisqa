# Quantum LIMIT-Graph egg - Quick Start Guide

Get up and running with the federated orchestration system in 5 minutes.

## Prerequisites

- Rust 1.70+ (2021 edition)
- Cargo (comes with Rust)

### Install Rust
```bash
# Linux/macOS
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Windows
# Download from https://rustup.rs/
```

## 1. Build the Project

```bash
cd quantum_integration/quantum-limit-graph-v2.4.0/rust/egg
cargo build --workspace
```

Expected output:
```
   Compiling limit-core v2.4.1
   Compiling limit-storage v2.4.1
   Compiling limit-orchestration v2.4.1
   Compiling limit-agents v2.4.1
   Compiling limit-api v2.4.1
    Finished dev [unoptimized + debuginfo] target(s)
```

## 2. Run Tests

```bash
cargo test --workspace
```

Expected output:
```
running 8 tests
test test_session_creation ... ok
test test_storage_persistence ... ok
test test_rd_series_persistence ... ok
test test_orchestrator_creation ... ok
test test_agent_creation ... ok
test test_multiple_sessions ... ok
test test_provenance_tracking ... ok
test test_governance_checkpoint ... ok

test result: ok. 8 passed; 0 failed
```

## 3. Run Examples

### Basic Session Example
```bash
cargo run --example basic_session
```

This demonstrates:
- Creating a session
- Persisting trace data
- Storing RD series (Reward-Difficulty curves)

### Federated Orchestration Example
```bash
cargo run --example federated_orchestration
```

This demonstrates:
- Creating multiple sessions
- Federated task distribution
- Governance policy enforcement

### Agent Benchmark Example
```bash
cargo run --example agent_benchmark
```

This demonstrates:
- Agent creation and configuration
- Performance benchmarking
- Serendipity tracking

## 4. Start the API Service

```bash
# Set log level
export RUST_LOG=info

# Run the API server
cargo run --bin limit-api
```

Expected output:
```
2025-11-25T14:00:00.000Z INFO limit_api: API listening on 0.0.0.0:8080
```

### Test the API

In another terminal:

```bash
# Health check
curl http://localhost:8080/health
# Expected: ok

# Create a session (when endpoints are implemented)
curl -X POST http://localhost:8080/sessions \
  -H "Content-Type: application/json" \
  -d '{"name": "my-session", "max_concurrency": 8}'
```

## 5. Use in Your Code

### Add as Dependency

In your `Cargo.toml`:
```toml
[dependencies]
limit-core = { path = "path/to/egg/crates/limit-core" }
limit-storage = { path = "path/to/egg/crates/limit-storage" }
limit-orchestration = { path = "path/to/egg/crates/limit-orchestration" }
limit-agents = { path = "path/to/egg/crates/limit-agents" }
```

### Basic Usage

```rust
use limit_core::{Session, SessionConfig};
use limit_storage::FileStorage;
use limit_orchestration::{Orchestrator, GovernancePolicy};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Create session
    let session = Session::new(SessionConfig {
        name: "my-research".into(),
        max_concurrency: 8,
        allow_network: true,
    });

    // Create storage
    let storage = FileStorage {
        root: "data".into(),
    };

    // Create orchestrator
    let orchestrator = Orchestrator::new(storage, GovernancePolicy {
        block_unsafe_merge: true,
        require_provenance: true,
    });

    // Use the system...
    println!("Session created: {}", session.id);
    
    Ok(())
}
```

## Common Tasks

### Enable Debug Logging
```bash
export RUST_LOG=debug
cargo run --example basic_session
```

### Run Specific Tests
```bash
# Test a specific crate
cargo test -p limit-core

# Test a specific function
cargo test test_session_creation
```

### Build for Production
```bash
cargo build --release --workspace
```

The optimized binaries will be in `target/release/`.

### Generate Documentation
```bash
cargo doc --workspace --open
```

This opens the generated documentation in your browser.

## Project Structure

```
egg/
├── crates/
│   ├── limit-core/          # Core types and session management
│   ├── limit-storage/       # Storage with provenance
│   ├── limit-orchestration/ # Federated orchestration
│   └── limit-agents/        # Modular agents
├── services/
│   └── api/                 # REST API service
├── examples/                # Usage examples
├── tests/                   # Integration tests
└── Cargo.toml              # Workspace configuration
```

## Configuration

### Environment Variables

```bash
# Log level (trace, debug, info, warn, error)
export RUST_LOG=info

# API server bind address
export BIND_ADDRESS=127.0.0.1:8080

# Storage root directory
export STORAGE_ROOT=/var/lib/limit
```

### Create Config File (Optional)

Create `config.toml`:
```toml
[server]
bind_address = "0.0.0.0:8080"
workers = 4

[storage]
type = "file"
root = "./data"

[backends]
default = "local"
```

## Troubleshooting

### Build Errors

**Problem**: Compilation errors
```bash
# Clean and rebuild
cargo clean
cargo build --workspace
```

**Problem**: Missing dependencies
```bash
# Update dependencies
cargo update
```

### Runtime Errors

**Problem**: Storage directory not found
```bash
# Create data directory
mkdir -p data
```

**Problem**: Port already in use
```bash
# Use different port
export BIND_ADDRESS=127.0.0.1:3000
```

### Test Failures

**Problem**: Tests fail due to existing data
```bash
# Clean test data
rm -rf data/test
cargo test --workspace
```

## Next Steps

1. **Read the Architecture**: See `FEDERATED_ARCHITECTURE.md` for detailed design
2. **Explore Examples**: Check `examples/` directory for more use cases
3. **Review Tests**: Look at `tests/` for integration patterns
4. **Extend the System**: Add custom agents, storage backends, or runners

## Getting Help

- **Documentation**: Run `cargo doc --workspace --open`
- **Examples**: Check `examples/` directory
- **Tests**: Review `tests/` for usage patterns
- **Architecture**: Read `FEDERATED_ARCHITECTURE.md`

## Performance Tips

1. **Use Release Mode**: `cargo build --release` for production
2. **Tune Concurrency**: Adjust `max_concurrency` in SessionConfig
3. **Monitor Metrics**: Use the `/metrics` endpoint
4. **Profile**: Use `cargo flamegraph` for performance analysis

## Security Checklist

- [ ] Use TLS in production
- [ ] Configure authentication
- [ ] Set up authorization policies
- [ ] Enable audit logging
- [ ] Restrict network access
- [ ] Use secure storage backend

---

**Ready to go!** You now have a working federated orchestration system.

For more details, see:
- `README.md` - Full documentation
- `FEDERATED_ARCHITECTURE.md` - Architecture details
- `IMPLEMENTATION_COMPLETE.md` - Completion status
