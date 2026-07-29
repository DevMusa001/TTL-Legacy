# Contract Upgrade Mechanism - Issue #1120

## Overview
This document describes the safe contract upgrade mechanism with timelock protection implemented for the TTL Vault contract.

## Implementation Details

### Constants (in lib.rs)
```rust
/// Contract upgrade timelock - Issue #1120
/// 72-hour time-lock before upgrade can be executed (in seconds)
pub const UPGRADE_TIMELOCK: u64 = 259_200; // 72 hours
```

### Data Types (in types.rs)

#### UpgradeProposal Structure
```rust
#[contracttype]
#[derive(Clone)]
pub struct UpgradeProposal {
    /// Hash of the new WASM bytecode
    pub new_wasm_hash: Bytes,
    /// Address of the admin who proposed the upgrade
    pub proposed_by: Address,
    /// Timestamp when the upgrade was proposed
    pub proposed_at: u64,
    /// Timestamp when the upgrade can be executed (proposed_at + 72 hours)
    pub executable_at: u64,
}
```

#### DataKey Variant
```rust
enum DataKey {
    // ...
    PendingUpgrade,  // Issue #1120: timelock-gated contract upgrade
}
```

#### Event Topics
```rust
pub const UPGRADE_PROPOSED_TOPIC: Symbol = symbol_short!("upg_prop");
pub const UPGRADE_EXECUTED_TOPIC: Symbol = symbol_short!("upg_exec");
pub const UPGRADE_CANCELLED_TOPIC: Symbol = symbol_short!("upg_canc");
```

#### Error Codes
```rust
pub enum ContractError {
    // ...
    NoPendingUpgrade = 92,         // Issue #1120: Contract upgrade mechanism
    UpgradeTimelocked = 93,        // Issue #1120: Upgrade not yet executable
    UpgradeInvalidWasm = 94,       // Issue #1120: Invalid WASM hash
}
```

### Core Functions

#### 1. propose_upgrade(env, new_wasm_hash)
**Purpose**: Admin proposes a new contract version with 72-hour timelock

**Authorization**: Admin-only

**Parameters**:
- `env`: Soroban environment
- `new_wasm_hash`: Bytes - SHA256 hash of the new WASM bytecode

**Returns**: Result<u64, ContractError>
- Success: timestamp when upgrade becomes executable
- Errors:
  - `NotAdmin` - Caller is not the admin
  - `UpgradeInvalidWasm` - Invalid WASM hash format (empty bytes)
  - `Paused` - Contract is paused

**Behavior**:
1. Verify caller is admin (require_auth)
2. Validate WASM hash is not empty
3. If pending upgrade exists, replace it
4. Create UpgradeProposal with:
   - `new_wasm_hash`: provided hash
   - `proposed_by`: admin address
   - `proposed_at`: current ledger timestamp
   - `executable_at`: proposed_at + 72 hours
5. Store in PendingUpgrade key
6. Emit UPGRADE_PROPOSED_TOPIC event with (admin, new_wasm_hash, executable_at)
7. Extend instance TTL
8. Return executable_at

#### 2. execute_upgrade(env)
**Purpose**: Execute pending upgrade after timelock expires

**Authorization**: Admin-only

**Parameters**:
- `env`: Soroban environment

**Returns**: Result<(), ContractError>

**Errors**:
- `NotAdmin` - Caller is not the admin
- `NoPendingUpgrade` - No pending upgrade exists
- `UpgradeTimelocked` - Timelock not yet elapsed
- `Paused` - Contract is paused

**Behavior**:
1. Verify caller is admin (require_auth)
2. Load pending upgrade or return NoPendingUpgrade
3. Check if current timestamp >= executable_at
   - If not, return UpgradeTimelocked
4. Call env.deployer().update_current_contract_wasm(new_wasm_hash)
5. Clear pending upgrade from storage
6. Emit UPGRADE_EXECUTED_TOPIC event with (admin, new_wasm_hash)
7. Log audit entry: "execute_upgrade"
8. Extend instance TTL
9. Return Ok(())

#### 3. cancel_upgrade(env)
**Purpose**: Admin cancels pending upgrade

**Authorization**: Admin-only

**Parameters**:
- `env`: Soroban environment

**Returns**: Result<(), ContractError>

**Errors**:
- `NotAdmin` - Caller is not the admin
- `NoPendingUpgrade` - No pending upgrade exists
- `Paused` - Contract is paused

**Behavior**:
1. Verify caller is admin (require_auth)
2. Load pending upgrade or return NoPendingUpgrade
3. Store cancelled wasm hash for audit trail
4. Remove pending upgrade from storage
5. Emit UPGRADE_CANCELLED_TOPIC event with (admin, wasm_hash)
6. Log audit entry: "cancel_upgrade"
7. Extend instance TTL
8. Return Ok(())

#### 4. get_pending_upgrade(env)
**Purpose**: Query pending upgrade details

**Authorization**: Public

**Parameters**:
- `env`: Soroban environment

**Returns**: Option<UpgradeProposal>

**Behavior**:
1. Retrieve pending upgrade from PendingUpgrade key
2. Return Some(upgrade) if exists, None otherwise

## Test Coverage

### Happy Path
- [x] Propose upgrade as admin → stores proposal with correct timelock
- [x] Query pending upgrade → returns proposal details
- [x] Wait 72+ hours → time-lock elapsed
- [x] Execute upgrade → contract updates to new WASM
- [x] Query upgrade after execution → None (cleared)

### Time-Lock Enforcement
- [x] Propose upgrade
- [x] Try to execute immediately → UpgradeTimelocked error
- [x] Try to execute before 72 hours → UpgradeTimelocked error
- [x] Execute after exactly 72 hours → success

### Cancellation
- [x] Propose upgrade as admin
- [x] Cancel by admin → successful cancellation
- [x] Verify UPGRADE_CANCELLED_TOPIC emitted
- [x] Try to execute after cancellation → NoPendingUpgrade error

### Replacement
- [x] Propose upgrade with hash A
- [x] Propose another upgrade with hash B → replaces A
- [x] Verify pending upgrade has hash B

### Authorization
- [x] Non-admin tries to propose → NotAdmin error
- [x] Non-admin tries to execute → NotAdmin error
- [x] Non-admin tries to cancel → NotAdmin error

### Edge Cases
- [x] Empty WASM hash rejection
- [x] Execute with no pending upgrade → NoPendingUpgrade error
- [x] Cancel with no pending upgrade → NoPendingUpgrade error
- [x] Paused contract rejects upgrade operations

## Events

### UPGRADE_PROPOSED_TOPIC
**Data**: (admin: Address, new_wasm_hash: Bytes, executable_at: u64)

### UPGRADE_EXECUTED_TOPIC
**Data**: (admin: Address, new_wasm_hash: Bytes)

### UPGRADE_CANCELLED_TOPIC
**Data**: (admin: Address, new_wasm_hash: Bytes)

## Audit Trail
All upgrade operations are logged with:
- Action: "propose_upgrade" | "execute_upgrade" | "cancel_upgrade"
- Caller: Admin address
- Timestamp: Ledger timestamp
- Details: WASM hash, executable timestamp, etc.

## Security Considerations

1. **72-Hour Timelock**: Provides users with time to:
   - Review proposed changes
   - Exit vaults if they don't trust the upgrade
   - Migrate to alternative contracts

2. **One Proposal at a Time**: Only one pending upgrade exists
   - Prevents confusion about which upgrade will execute
   - Clear upgrade path

3. **Admin-Only**: Only contract admin can propose/execute/cancel
   - Prevents unauthorized upgrades
   - Clear authority chain

4. **Pausable**: Upgrade operations blocked when contract is paused
   - Prevents upgrades during security incidents
   - Allows emergency pause to prevent bad upgrades

5. **Audit Trail**: All operations logged
   - Transparent history of upgrades
   - Forensic capability

## Implementation Notes

- Uses Soroban's `env.deployer().update_current_contract_wasm()` for actual upgrade
- Timelock stored explicitly (not calculated) for clarity
- Pending upgrade keys are instance-level for efficiency
- All times are ledger timestamps (UTC seconds on Stellar)
- TTL extended on all operations to maintain persistence
