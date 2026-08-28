# Contributing to TTL-Legacy

We welcome contributions! Please follow these guidelines to help us maintain project quality.

## Getting Started

### Prerequisites

| Tool | Version | Notes |
|---|---|---|
| Rust | 1.70+ | Install via [rustup](https://rustup.rs) |
| Node.js | 18+ | Required for frontend dev tooling |
| Docker | 24+ | Required for local dev stack |
| Stellar CLI | latest | `cargo install stellar-cli --locked` |
| Soroban CLI | latest | Bundled with Stellar CLI |
| `just` | any | Optional but recommended — `cargo install just` |

### Local Dev Setup (Step by Step)

1. **Copy the environment file:**
   ```bash
   cp .env.example .env
   ```

2. **Start the local dev stack:**
   ```bash
   docker-compose up -d
   ```
   This starts PostgreSQL (port 5432), the backend (port 3000), and Stellar Quickstart (port 8000).
   Wait for health checks to pass before proceeding.

3. **Verify the backend is healthy:**
   ```bash
   curl http://localhost:3000/health
   ```
   You should receive a `200 OK` response.

4. **Build the Soroban contracts:**
   ```bash
   just build
   # or without just:
   ./scripts/build.sh
   ```

5. **Deploy to the local Stellar Quickstart node:**
   ```bash
   just deploy-testnet
   ```
   This deploys to the local node at `localhost:8000` as configured in `docker-compose.override.yml`.

6. **Start the frontend dev server:**
   ```bash
   cd frontend && npm run dev
   ```
   The frontend dev server runs at `http://localhost:5173`.

### Running the Backend Against the Local Node

In your `.env`, set:
```env
STELLAR_RPC_URL=http://localhost:8000
```

This points the backend at your local Stellar Quickstart instance instead of testnet.

## Development Workflow
1. **Fork the repo** and create your branch: git checkout -b feature/your-feature-name.
2. **Formatting:** We use rustfmt. Please run the following command before committing:
   ```bash
   just fmt
   # or without just:
   cargo fmt
   ```
3. **Testing:** Run the test suite before submitting:
   ```bash
   just test
   # or without just:
   ./scripts/test.sh
   ```
4. **Pre-PR check:** Run the full CI suite locally before opening a PR:
   ```bash
   just ci
   ```
5. **Pull Requests:** Open a PR against main. Ensure your PR description clearly outlines the changes and links to the relevant issue.

## Automated Security Scanning in CI

The CI pipeline includes three layers of automated security scanning that run on every push and pull request:

### 1. **Gitleaks (Secret Detection)**
Gitleaks scans for accidentally committed secrets (API keys, tokens, credentials, etc.) in the git history and working directory.

- **Configuration:** `.gitleaks.toml` contains the detection rules and allowlist for test fixtures
- **False positives:** If legitimate test credentials or documentation examples trigger a false positive, add them to the `allowlist.regexes` or `allowlist.paths` in `.gitleaks.toml`
- **Local testing:** Run gitleaks locally before pushing:
  ```bash
  gitleaks detect --source . --config .gitleaks.toml --redact --fail
  ```

### 2. **Cargo Audit (Vulnerability Advisories)**
Cargo audit checks Rust dependencies against a database of known security vulnerabilities.

- **Runs:** `cargo audit --deny warnings`
- **What it catches:** Known CVEs in transitive dependencies
- **If audit fails:** Update the vulnerable dependency to a patched version, or if no fix is available, document the exception and work with the security team
- **Local testing:**
  ```bash
  cargo audit --deny warnings
  ```

### 3. **Clippy Linting (Static Analysis)**
Clippy enforces lint warnings as hard errors to catch common mistakes and anti-patterns.

- **Runs:** `cargo clippy --package ttl-vault -- -D warnings`
- **Warnings treated as errors:** All clippy warnings must be fixed before merging
- **Suppress spurious warnings:** Use `#[allow(clippy::rule_name)]` on specific code with a comment explaining why
- **Local testing:**
  ```bash
  cargo clippy --package ttl-vault -- -D warnings
  ```

### Running Security Checks Locally

Before opening a PR, run all security checks locally:

```bash
# Run the full CI suite including all security scans
just ci

# Or manually:
cargo fmt --all -- --check
cargo clippy --package ttl-vault -- -D warnings
cargo audit --deny warnings
gitleaks detect --source . --config .gitleaks.toml --redact --fail
```

If any check fails, fix it before pushing. The CI pipeline will enforce the same checks.

## Available `just` Targets

Install [just](https://just.systems/man/en/packages.html), then run `just --list` from the repo root:

```
Available recipes:
    audit               # Run cargo-audit (install with: cargo install cargo-audit)
    build               # Build both Soroban contracts for wasm32 release
    ci                  # Run build + test + clippy in one shot (useful before opening a PR)
    clippy              # Run clippy (warnings treated as errors, matching CI)
    deploy-mainnet      # Deploy to Stellar mainnet (requires STELLAR_MAINNET_RPC_URL; prompts for confirmation)
    deploy-mainnet-force# Force-redeploy to mainnet without the existing-contract prompt
    deploy-testnet      # Deploy to Stellar testnet (prompts if a contract already exists)
    deploy-testnet-force# Force-redeploy to testnet without confirmation prompt
    docker-down         # Stop and remove local dev stack containers
    docker-up           # Start local dev stack (PostgreSQL, backend, Stellar Quickstart)
    env-setup           # Copy .env.example to .env (skips if .env already exists)
    fmt                 # Auto-format all code
    fmt-check           # Check code formatting
    test                # Run the full ttl_vault test suite
```

## Fuzz Testing

The TTL Vault contract includes comprehensive fuzz testing to catch panics, unexpected errors, and boundary condition violations. Fuzz tests are located in `contracts/ttl_vault/fuzz/`.

### Running Fuzz Tests Locally

Fuzz testing requires the nightly Rust toolchain:

```bash
# Install nightly if you haven't already
rustup install nightly
rustup component add rust-src --toolchain nightly
```

Run a single fuzz target:

```bash
cd contracts/ttl_vault/fuzz
# Fuzz for 10 minutes (600 seconds)
cargo +nightly fuzz run fuzz_create_vault -- -max_total_time=600

# Fuzz with corpus (if available)
cargo +nightly fuzz run fuzz_deposit corpus/fuzz_deposit -- -max_total_time=600
```

Run all fuzz targets:

```bash
# Run each target for 10 minutes
for target in fuzz_vesting fuzz_create_vault fuzz_deposit fuzz_withdraw fuzz_check_in; do
  cargo +nightly fuzz run $target -- -max_total_time=600
done
```

### Fuzz Targets

Four primary contract entry points have fuzz targets:

- **`fuzz_create_vault`** - Tests vault creation with arbitrary addresses and intervals
- **`fuzz_deposit`** - Tests deposits with arbitrary amounts and limits
- **`fuzz_withdraw`** - Tests withdrawals with approval thresholds and guards
- **`fuzz_check_in`** - Tests check-ins with TTL caps and inactivity penalties
- **`fuzz_vesting`** - Tests vesting schedule calculations (existing)

### Handling Fuzz Findings

If the fuzzer finds a crash or panic:

1. **Reproduce locally**: The fuzzer creates a corpus file with the failing input
2. **Minimize**: Use `cargo +nightly fuzz cmin <target>` to reduce input size
3. **Fix**: Address the underlying issue in the contract code
4. **Verify**: Re-run the fuzzer to confirm the fix
5. **Commit**: Add the test case to prevent regressions

Example:
```bash
# If fuzz_create_vault crashes
cd contracts/ttl_vault/fuzz
# Reproduce the crash
cargo +nightly fuzz run fuzz_create_vault corpus/fuzz_create_vault/crash-xxx
# Minimize the input
cargo +nightly fuzz cmin fuzz_create_vault corpus/fuzz_create_vault/crash-xxx
```

### CI Fuzz Testing

Fuzz tests run nightly via `.github/workflows/nightly-fuzz.yml`:

- **Schedule**: Daily at 2 AM UTC
- **Duration**: 10 minutes per target (configurable)
- **Failure Mode**: Any crash/panic causes CI failure
- **Artifacts**: Corpus and crash artifacts uploaded for 30 days

Manual trigger:
```bash
# Trigger via GitHub CLI with custom parameters
gh workflow run nightly-fuzz.yml -f fuzz_time=1800 -f max_len=2048
```

### Corpus Management

Seed corpus files are stored in `contracts/ttl_vault/fuzz/corpus/`:

- `fuzz_create_vault/` - Seeds for vault creation testing
- `fuzz_deposit/` - Seeds for deposit testing
- `fuzz_withdraw/` - Seeds for withdrawal testing
- `fuzz_check_in/` - Seeds for check-in testing

Generate or regenerate corpus:
```bash
cd contracts/ttl_vault/fuzz
python3 generate_corpus.py
```

After fuzzing discovers new interesting inputs, commit them:
```bash
git add contracts/ttl_vault/fuzz/corpus/
git commit -m "Update fuzz corpus after extended fuzzing run"
```

### Best Practices

1. **Run fuzz tests before major releases** - Extended runs (hours/days) catch subtle bugs
2. **Commit crash inputs** - Prevents regressions
3. **Review fuzzer findings** - Understand why an input triggered coverage
4. **Maintain corpus** - Keep seed inputs diverse and minimal

For more details, see [contracts/ttl_vault/fuzz/README.md](contracts/ttl_vault/fuzz/README.md).

## Troubleshooting FAQ

### Docker containers not starting

Check logs and verify ports are free:
```bash
docker-compose logs
```
Ensure nothing else is bound to ports **5432** (PostgreSQL), **3000** (backend), or **8000** (Stellar Quickstart).
If a port is already in use, stop the conflicting process or change the port mapping in `docker-compose.override.yml`.

### `cargo build` fails with `wasm32` target missing

The `wasm32-unknown-unknown` target must be added to your Rust toolchain:
```bash
rustup target add wasm32-unknown-unknown
```

### Stellar CLI `account not found`

Your deployer account doesn't exist on the network yet. Fund it via Friendbot (testnet only):
```bash
curl "https://friendbot.stellar.org?addr=<YOUR_ADDRESS>"
```
Replace `<YOUR_ADDRESS>` with the public key printed by `stellar keys address <key-name>`.

### Backend fails to start: `DATABASE_URL not set`

You haven't configured the environment file:
```bash
cp .env.example .env
```
Open `.env` and fill in the `DATABASE_URL` and other required values. The example file documents each variable.

### Frontend cannot reach backend

Verify that `VITE_API_BASE_URL` in your `.env` matches the port the backend is running on (default: `http://localhost:3000`). Restart the frontend dev server after changing `.env` values.

## OpenAPI Spec Update Workflow

All changes to backend routes **must** be accompanied by a corresponding update to
`docs/openapi.yaml`. This is enforced in CI via `openapi-spec-validator`.

### Steps When Adding or Changing a Route

1. **Implement the route** in `backend/src/routes.rs` and `backend/src/handlers.rs`.
2. **Update `docs/openapi.yaml`** — add or modify the path, method, request body,
   parameters, and response schemas.
3. **Update `backend/tests/openapi_contract_test.rs`** — add the new route to the
   `ROUTES` constant.
4. **Run validation locally**:
   ```bash
   python3 -m pip install openapi-spec-validator
   python3 -m openapi_spec_validator docs/openapi.yaml
   ```
5. **Open your PR** — CI will validate the spec automatically.

### Why This Matters

A stale OpenAPI spec misleads API consumers (mobile apps, third-party integrations)
and breaks contract tests. Keeping the spec in sync is a first-class requirement,
not a nice-to-have.

### Spec Linting

The CI pipeline runs `openapi-spec-validator` against `docs/openapi.yaml` on every
push and pull request. A failing validation blocks merge.

## Style Guide
- Follow standard Rust idiomatic practices.
- Use /// for all public function documentation.
- Maintain consistency with the existing project structure.
