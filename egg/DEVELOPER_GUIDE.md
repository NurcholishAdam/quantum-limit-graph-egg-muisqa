# Quantum LIMIT-Graph Egg - Developer Guide

## 🎯 Quick Reference

This guide helps you understand which modules to use for different tasks.

---

## 📦 Module Selection Guide

### Need to manage sessions?
→ Use **`limit-core`**
```rust
use limit_core::{Session, SessionConfig};

let session = Session::new(SessionConfig {
    name: "my-session".into(),
    max_concurrency: 4,
    allow_network: false,
});
```

### Need to store data?
→ Use **`limit-storage`**
```rust
use limit_storage::FileStorage;

let storage = FileStorage {
    root: "data/sessions".into(),
};

storage.persist_trace(session_id, trace_id, data).await?;
```

### Need to orchestrate multiple sessions?
→ Use **`limit-orchestration`**
```rust
use limit_orchestration::{Orchestrator, GovernancePolicy};

let orchestrator = Orchestrator::new(storage, GovernancePolicy {
    block_unsafe_merge: true,
    require_provenance: true,
});
```

### Need to create agents?
→ Use **`limit-agents`**
```rust
use limit_agents::{Agent, AgentConfig};

let agent = Agent::new(AgentConfig {
    name: "my-agent".into(),
    timeout_ms: 5000,
});
```

### Need to expose an API?
→ Use **`services/api`** as template

---

## 🔄 Common Workflows

### Workflow 1: Basic Session with Storage

```rust
use limit_core::{Session, SessionConfig, TraceId};
use limit_storage::FileStorage;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Create session
    let session = Session::new(SessionConfig {
        name: "workflow-1".into(),
        max_concurrency: 4,
        allow_network: false,
    });

    // 2. Create storage
    let storage = FileStorage {
        root: "data/workflow-1".into(),
    };

    // 3. Do work and persist
    let trace_id = TraceId::new();
    let data = serde_json::json!({
        "task": "example",
        "result": 42,
    });

    storage.persist_trace(session.id, trace_id, data).await?;

    Ok(())
}
```

### Workflow 2: Agent Benchmarking

```rust
use limit_agents::{Agent, AgentConfig, BenchmarkRun};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Create agent
    let agent = Agent::new(AgentConfig {
        name: "benchmark-agent".into(),
        timeout_ms: 5000,
    });

    // 2. Run tasks and collect metrics
    let start = chrono::Utc::now();
    // ... run tasks ...
    let end = chrono::Utc::now();

    // 3. Create benchmark report
    let benchmark = BenchmarkRun {
        agent_name: agent.config.name.clone(),
        start_time: start,
        end_time: end,
        total_tasks: 100,
        successful_tasks: 95,
        failed_tasks: 5,
        avg_latency_ms: 42.5,
        p95_latency_ms: 85.0,
        p99_latency_ms: 120.0,
    };

    println!("Benchmark: {:?}", benchmark);

    Ok(())
}
```

### Workflow 3: Federated Orchestration

```rust
use limit_core::{Session, SessionConfig, BackendRunner, RunnerKind};
use limit_storage::FileStorage;
use limit_orchestration::{Orchestrator, GovernancePolicy};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Create storage
    let storage = FileStorage {
        root: "data/federated".into(),
    };

    // 2. Create orchestrator with governance
    let orchestrator = Orchestrator::new(storage, GovernancePolicy {
        block_unsafe_merge: true,
        require_provenance: true,
    });

    // 3. Create multiple sessions
    let sessions: Vec<Session> = (0..3)
        .map(|i| {
            Session::new(SessionConfig {
                name: format!("session-{}", i),
                max_concurrency: 4,
                allow_network: false,
            })
        })
        .collect();

    // 4. Create backend runner
    let runner = BackendRunner::new(RunnerKind::Local);

    // 5. Execute tasks across sessions
    for session in &sessions {
        // ... execute tasks ...
    }

    Ok(())
}
```

---

## 🧩 Module Dependencies

```
services/api
    ↓
limit-orchestration
    ↓ ↓
limit-agents  limit-storage
    ↓             ↓
    limit-core ←──┘
```

**Rule**: Always depend on `limit-core` first, then add other crates as needed.

---

## 📝 Adding a New Crate

### Step 1: Create crate structure
```bash
mkdir -p crates/my-crate/src
```

### Step 2: Create Cargo.toml
```toml
[package]
name = "my-crate"
version.workspace = true
edition.workspace = true
authors.workspace = true
license.workspace = true

[dependencies]
limit-core = { path = "../limit-core" }
# Add other dependencies
```

### Step 3: Create src/lib.rs
```rust
pub mod my_module;

pub use my_module::MyType;
```

### Step 4: Add to workspace
Edit root `Cargo.toml`:
```toml
[workspace]
members = [
    # ... existing members ...
    "crates/my-crate",
]
```

### Step 5: Create example
```rust
// examples/my_example.rs
use my_crate::MyType;

fn main() {
    println!("Example using my-crate");
}
```

---

## 🧪 Testing Guidelines

### Unit Tests
Place in the same file as the code:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_my_function() {
        assert_eq!(my_function(), expected);
    }

    #[tokio::test]
    async fn test_async_function() {
        let result = async_function().await;
        assert!(result.is_ok());
    }
}
```

### Integration Tests
Place in `tests/` directory:
```rust
// tests/integration_test.rs
use limit_core::Session;
use limit_storage::FileStorage;

#[tokio::test]
async fn test_session_with_storage() {
    // Test cross-crate functionality
}
```

### Running Tests
```bash
# All tests
cargo test --workspace

# Specific crate
cargo test -p limit-core

# Specific test
cargo test test_name

# With output
cargo test -- --nocapture
```

---

## 🔍 Debugging Tips

### Enable Tracing
```rust
tracing_subscriber::fmt()
    .with_env_filter("debug")  // or "trace" for more detail
    .init();
```

### Environment Variables
```bash
# Set log level
export RUST_LOG=debug

# Enable backtraces
export RUST_BACKTRACE=1

# Full backtraces
export RUST_BACKTRACE=full
```

### Debugging in Examples
```bash
# Run with debug output
RUST_LOG=debug cargo run --example basic_session

# Run with backtrace
RUST_BACKTRACE=1 cargo run --example basic_session
```

---

## 🚀 Performance Tips

### 1. Use Release Mode
```bash
cargo build --release
cargo run --release --example my_example
```

### 2. Profile with Flamegraph
```bash
cargo install flamegraph
cargo flamegraph --example my_example
```

### 3. Benchmark with Criterion
```toml
[dev-dependencies]
criterion = "0.5"

[[bench]]
name = "my_benchmark"
harness = false
```

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_function(c: &mut Criterion) {
    c.bench_function("my_function", |b| {
        b.iter(|| my_function(black_box(42)))
    });
}

criterion_group!(benches, benchmark_function);
criterion_main!(benches);
```

---

## 🐳 Docker Development

### Build Image
```bash
docker build -t quantum-limit-graph-egg:dev .
```

### Run Container
```bash
docker run -p 8080:8080 quantum-limit-graph-egg:dev
```

### Development with Docker Compose
```bash
# Start services
docker-compose up

# Rebuild and start
docker-compose up --build

# Stop services
docker-compose down

# View logs
docker-compose logs -f
```

---

## 📚 Documentation

### Generate Docs
```bash
# Generate and open docs
cargo doc --open --workspace

# Include private items
cargo doc --document-private-items --workspace
```

### Doc Comments
```rust
/// This is a doc comment for a public item.
///
/// # Examples
///
/// ```
/// use my_crate::my_function;
/// assert_eq!(my_function(42), 84);
/// ```
///
/// # Errors
///
/// Returns an error if the input is negative.
pub fn my_function(x: i32) -> Result<i32> {
    // ...
}
```

---

## 🔧 Common Commands

```bash
# Check code without building
cargo check --workspace

# Format code
cargo fmt --all

# Lint code
cargo clippy --workspace -- -D warnings

# Update dependencies
cargo update

# Clean build artifacts
cargo clean

# Show dependency tree
cargo tree

# Audit dependencies for security issues
cargo audit
```

---

## 🎓 Best Practices

### 1. Error Handling
```rust
use anyhow::{Result, Context};

fn my_function() -> Result<()> {
    some_operation()
        .context("Failed to perform operation")?;
    Ok(())
}
```

### 2. Async Functions
```rust
use async_trait::async_trait;

#[async_trait]
pub trait MyTrait {
    async fn my_method(&self) -> Result<()>;
}
```

### 3. Configuration
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MyConfig {
    pub name: String,
    pub timeout_ms: u64,
}
```

### 4. Logging
```rust
use tracing::{info, warn, error, debug, trace};

info!("Starting operation");
debug!("Debug details: {:?}", data);
warn!("Warning: {}", message);
error!("Error occurred: {}", err);
```

---

## 🆘 Troubleshooting

### Problem: Compilation errors
**Solution**: 
```bash
cargo clean
cargo build --workspace
```

### Problem: Test failures
**Solution**:
```bash
# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_name -- --nocapture
```

### Problem: Dependency conflicts
**Solution**:
```bash
# Update Cargo.lock
cargo update

# Check dependency tree
cargo tree
```

### Problem: Slow builds
**Solution**:
```bash
# Use cargo-watch for incremental builds
cargo install cargo-watch
cargo watch -x check

# Enable parallel compilation
export CARGO_BUILD_JOBS=8
```

---

## 📞 Getting Help

1. **Check Examples**: Look at `examples/` directory
2. **Read Docs**: Run `cargo doc --open`
3. **Check Tests**: Look at test cases for usage patterns
4. **Run Validation**: `python validate_egg_structure.py`

---

## ✅ Checklist for New Features

- [ ] Add code to appropriate crate
- [ ] Write unit tests
- [ ] Add integration test if needed
- [ ] Create example demonstrating usage
- [ ] Update documentation
- [ ] Run `cargo fmt`
- [ ] Run `cargo clippy`
- [ ] Run `cargo test --workspace`
- [ ] Update CHANGELOG.md
- [ ] Update README.md if needed

---

**Happy Coding! 🚀**
