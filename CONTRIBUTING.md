# Contributing to TTL-Legacy

We welcome contributions! Please follow these guidelines to help us maintain project quality.

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

## Style Guide
- Follow standard Rust idiomatic practices.
- Use /// for all public function documentation.
- Maintain consistency with the existing project structure.
