# PR: Fix partial_release Auth Check & Add OpenTelemetry Distributed Tracing

## Summary

This PR addresses two issues:

1. **Fix #792**: Audits and hardens the `partial_release` authorization check, ensuring only the vault owner can execute it. Adds a test verifying non-owner rejection.
2. **Fix #1145**: Instruments key backend functions with OpenTelemetry spans across the notification pipeline, scheduler, and route handlers for end-to-end distributed tracing.

---

## Changes

### Issue #792 — Fix `partial_release` Auth Check (`partial_release` Does Not Verify Caller Is Owner)

- **Audited `partial_release` auth check** at `contracts/ttl_vault/src/lib.rs:3049` — confirmed it correctly uses `vault.owner.require_auth()` (the explicit owner check, not a generic `require_auth()`).
- **Added non-owner rejection test** (`test_partial_release_rejects_non_owner`) in `contracts/ttl_vault/src/test.rs`:
  - Creates a vault as the owner and deposits funds.
  - Replaces the blanket `mock_all_auths` with a specific auth entry for an attacker address.
  - Verifies `try_partial_release` returns an error when the caller is not the vault owner.

### Issue #1145 — Add OpenTelemetry Distributed Tracing to Backend

All required OpenTelemetry crates (`opentelemetry`, `opentelemetry-sdk`, `opentelemetry-otlp`, `opentelemetry-semantic-conventions`, `tracing-opentelemetry`) were already present in `backend/Cargo.toml`. The module `backend/src/otel.rs` already provided `init_tracer`, `try_init_tracer`, `OtelGuard`, and `stellar_rpc_span`.

This PR adds `#[instrument]` spans to the following previously uninstrumented functions:

#### Notification Dispatch (`backend/src/notifications.rs`)
- `schedule_expiry_warning` — records `vault_id`
- `schedule_immediate` — records `vault_id`, `notification_type`
- `flush_pending` — background dispatch of due notifications
- `flush_retries` — retry of failed notifications
- `deliver_with_retry` — records `vault_id`, `notification_type`, `attempt`

#### Scheduler (`backend/src/scheduler.rs`)
- `run` — main scheduler loop (reminder + TTL insurance)
- `extend_ttl_for_inactive_owners` — TTL extension for inactive vault owners

#### Route Handlers (`backend/src/routes.rs`)
- `list_vault_reminders` — records `vault_id`
- `set_preferences` — records `vault_id`
- `get_preferences` — records `vault_id`
- `delete_preferences` — records `vault_id`
- `unsubscribe` — process unsubscribe requests
- `simulate_release` — records `vault_id`

#### Documentation (`docs/monitoring-guide.md`)
- Updated the **Instrumented Operations** table with all new spans and their attributes.

---

## Testing

- **Contract tests**: Added `test_partial_release_rejects_non_owner` — uses `env.set_auth()` to override blanket auth and verify that a non-owner caller is correctly rejected.
- **Backend**: All instrumentation is compile-time checked (macro expansion). No behavioral changes — spans are purely observability.

---

## Verification

- [x] Audit confirms `partial_release` uses `vault.owner.require_auth()`
- [x] New contract test validates non-owner rejection
- [x] All notification dispatch functions instrumented with spans
- [x] Scheduler functions instrumented
- [x] Route handlers instrumented
- [x] Monitoring guide updated with full span table

---

closes #792
closes #1145
