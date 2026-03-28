# Contributing to StellarForge

Thanks for your interest in contributing. This guide covers everything you need to get set up, run tests, meet code style requirements, and submit a pull request.

---

## Prerequisites & Local Setup

You will need the following tools installed before you can build or test the contracts.

### Rust

- **Edition:** 2021
- **Target:** `wasm32v1-none`

Install the required target:

```bash
rustup target add wasm32v1-none
```

If you don't have Rust installed, follow the official guide at <https://www.rust-lang.org/tools/install>.

### Stellar CLI

**v25.2.0 or higher** is required.

```bash
cargo install --locked stellar-cli
```

Full installation docs: <https://developers.stellar.org/docs/smart-contracts/getting-started/setup>

### Clone and Build

```bash
git clone https://github.com/your-org/stellarforge.git
cd stellarforge
cargo build --workspace
```

---

## Running Tests

Run the full test suite across all workspace members:

```bash
cargo test --workspace
```

Run tests for a single contract using the `-p` flag:

```bash
cargo test -p forge-governor
cargo test -p forge-multisig
cargo test -p forge-oracle
cargo test -p forge-stream
cargo test -p forge-vesting
```

All tests must pass before you submit a PR.

---

## Testing Philosophy

We believe comprehensive tests are essential for smart contract reliability. Good tests prevent bugs, document expected behavior, and give contributors confidence when making changes.

### What to Test

Every contract function should have tests covering:

1. **Happy paths** — Normal, expected usage with valid inputs
2. **Error paths** — Invalid inputs, unauthorized access, and edge cases that should fail gracefully
3. **Boundary conditions** — Limits, thresholds, and transition points (e.g., exactly at staleness boundary, one second past)
4. **State transitions** — Verify state changes persist correctly (e.g., price updates overwrite previous values)

### Test Structure and Naming

- Place tests in a `#[cfg(test)]` module at the bottom of `lib.rs`
- Name test functions descriptively: `test_<action>_<condition>_<expected_result>`
  - Good: `test_submit_price_with_zero_value_rejected`
  - Good: `test_get_price_at_exact_staleness_boundary_succeeds`
  - Avoid: `test_price`, `test_error_case`
- Use a `setup()` helper function to reduce boilerplate
- Group related tests with comments (e.g., `// ── Staleness boundary tests ──`)

### Using Soroban's Test Environment

Soroban provides powerful testing utilities. Key patterns:

```rust
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Env,
};

// Mock all authorization checks
env.mock_all_auths();

// Manipulate ledger time for staleness/expiry tests
env.ledger().with_mut(|l| l.timestamp = 1000);

// Generate test addresses
let admin = Address::generate(&env);

// Test error cases with try_ methods
let result = client.try_submit_price(&base, &quote, &0);
assert_eq!(result, Err(Ok(OracleError::InvalidPrice)));
```

### Example of a Well-Written Test

```rust
/// Verify that submitting a new price for an existing pair overwrites the old one.
/// This ensures stale prices are not retained.
#[test]
fn test_price_update_overwrites_previous_price() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, client) = setup(&env);

    let base = Symbol::new(&env, "XLM");
    let quote = Symbol::new(&env, "USDC");

    // Submit initial price at timestamp 1000
    env.ledger().with_mut(|l| l.timestamp = 1000);
    let initial_price = 10_000_000i128;
    client.submit_price(&base, &quote, &initial_price);

    // Verify initial price is stored
    let data = client.get_price(&base, &quote);
    assert_eq!(data.price, initial_price);
    assert_eq!(data.updated_at, 1000);

    // Submit new price for the same pair at timestamp 2000
    env.ledger().with_mut(|l| l.timestamp = 2000);
    let new_price = 15_000_000i128;
    client.submit_price(&base, &quote, &new_price);

    // Verify get_price() returns the new price, not the old one
    let data = client.get_price(&base, &quote);
    assert_eq!(data.price, new_price, "Expected new price to overwrite old price");
    assert_eq!(data.updated_at, 2000, "Expected timestamp to be updated");
}
```

This test demonstrates:
- Clear documentation explaining what's being tested
- Descriptive variable names and comments
- Testing both the action and its side effects
- Explicit assertions with helpful failure messages
- Time manipulation to test state changes

### When Adding New Tests

- If you're fixing a bug, add a test that would have caught it
- If you're adding a feature, test both success and failure cases
- If you're modifying existing behavior, update related tests
- Run `cargo test -p <contract-name>` frequently during development

---

## Code Style

### Formatting

```bash
cargo fmt --all
```

This must produce no changes. Run it before committing.

### Linting

```bash
cargo clippy --all-targets -- -D warnings
```

This must produce zero warnings.

### Additional Rules

- New public functions and types require `///` doc comments.
- No `unsafe` code is permitted in any contract.
- No external crate dependencies beyond `soroban-sdk` are permitted without prior discussion with maintainers.

---

## Pre-Commit Hook (Optional but Recommended)

A pre-commit hook can automatically check formatting and linting before each commit, catching issues early and saving CI time.

### Setting Up the Hook

**Quick Setup** (using the provided template):

```bash
cp scripts/pre-commit .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit
```

**Manual Setup**:

1. Create the hook file:

```bash
touch .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit
```

2. Add the following script to `.git/hooks/pre-commit`:

```bash
#!/bin/bash

echo "Running pre-commit checks..."

# Check formatting
echo "Checking code formatting..."
if ! cargo fmt --all -- --check; then
    echo "❌ Code formatting check failed. Run 'cargo fmt --all' to fix."
    exit 1
fi

# Run clippy
echo "Running clippy..."
if ! cargo clippy --all-targets -- -D warnings; then
    echo "❌ Clippy found issues. Fix them before committing."
    exit 1
fi

# Run tests (optional - can be slow for large projects)
# Uncomment the following lines if you want to run tests before each commit:
# echo "Running tests..."
# if ! cargo test --workspace; then
#     echo "❌ Tests failed. Fix them before committing."
#     exit 1
# fi

echo "✅ All pre-commit checks passed!"
exit 0
```

### Notes

- The hook is **optional** but highly recommended to catch issues before pushing
- If you need to bypass the hook in an emergency, use `git commit --no-verify`
- The test step is commented out by default since it can be slow; uncomment if desired
- Each contributor must set up the hook individually (hooks are not tracked by git)

---

## Benchmarking

StellarForge uses Soroban's built-in **budget tracking** to measure contract execution costs. Unlike wall-clock benchmarks, the Soroban budget reports deterministic, environment-independent metrics:

- **CPU instructions** — computational work performed by the contract host
- **Memory bytes** — heap allocations made during execution

These numbers are stable across machines and reflect the actual on-chain resource consumption that determines transaction fees.

### Running Benchmarks

```bash
make bench
# or manually:
cargo run -p forge-benches
```

### Example Output

```
StellarForge Contract Benchmarks
=================================
Metric: Soroban budget (CPU instructions + memory bytes)
Note:   Each measurement is reset before the call under test.

[forge-vesting]
  initialize()                        cpu=       52587 instructions   mem=      8278 bytes
  claim() at 50%                      cpu=      225128 instructions   mem=     34124 bytes
  claim() at 100%                     cpu=      229029 instructions   mem=     34143 bytes

[forge-oracle]
  submit_price()                      cpu=      167582 instructions   mem=     53256 bytes
  get_price()                         cpu=       66557 instructions   mem=     10774 bytes
  ...
```

### Interpreting Results

- **CPU instructions** — higher values mean more compute cost and higher fees. Watch for unexpected spikes after refactors.
- **Memory bytes** — heap usage per call. Large allocations (e.g. iterating over many stored entries) show up here.
- Each measurement is isolated: `env.budget().reset_default()` is called immediately before the function under test, so setup costs don't pollute results.

### Adding a New Benchmark

Open `benches/src/main.rs` and add a `bench_<contract>()` function following the existing pattern:

```rust
fn bench_mycontract(env: &Env) {
    println!("\n[forge-mycontract]");
    // ... setup ...
    env.budget().reset_default();
    client.my_function(&arg);
    print_budget("my_function()", env);
}
```

Then call it from `main()`. No external dependencies are needed — the benchmark binary uses the same `soroban-sdk` testutils already used by the test suite.

### Tracking Regressions

Run `make bench` before and after a change and compare the numbers. A significant increase in CPU instructions for a core function (e.g. >10%) warrants investigation before merging.

---

## Pull Request Process

1. Fork the repository and create a feature branch off `main`.
2. Make your changes, keeping commits logically atomic (or squash before opening the PR).
3. Ensure all CI checks pass locally before requesting review:
   - `cargo fmt --all` — no changes
   - `cargo clippy --all-targets -- -D warnings` — zero warnings
   - `cargo test --workspace` — all tests pass
4. Open a PR against `main`. Your PR description must summarise what changed and why.
5. If your PR introduces a new contract or public API, include tests covering error paths and state transitions.
6. **Breaking Changes:** If your PR introduces breaking changes (see [Versioning](README.md#-versioning)), you **must** include a migration guide. Use the template in [`docs/migrations/TEMPLATE.md`](docs/migrations/TEMPLATE.md) and place the guide in the `docs/migrations/` directory.
7. At least one maintainer approval is required before a PR is merged.

---

## License

By contributing, you agree that your contributions will be licensed under the [MIT License](LICENSE).
