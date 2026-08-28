# Threat Model & Security

## Threat Vectors

### 1. Owner Key Compromise

**Risk**: Attacker gains access to owner's private key

**Mitigations**:
- Passkey authentication (planned) eliminates seed phrase exposure
- Owner can update beneficiary before attacker triggers release
- Pause mechanism allows admin to freeze contract

### 2. Premature Release

**Risk**: Beneficiary triggers release before owner is deceased

**Mitigations**:
- `is_expired()` check enforces TTL expiry
- Returns `ContractError::NotExpired` if triggered early
- Owner can check in to reset countdown

### 3. Admin Abuse

**Risk**: Admin pauses contract or changes configuration maliciously

**Mitigations**:
- Admin cannot access vault funds
- Admin cannot change vault owners or beneficiaries
- Two-step admin transfer with `propose_admin` and `accept_admin`
- Transparent on-chain actions
- `accept_admin` emits an `adm_done` event with `(old_admin, new_admin, accepted_at)` so monitoring systems can detect admin rotations without inspecting raw ledger topics

### 4. Re-initialization Attack

**Risk**: Attacker re-initializes contract with new admin

**Mitigations**:
- `initialize()` checks for existing admin/token
- Returns `ContractError::AlreadyInitialized`
- Tested in `test_initialize_guard_against_double_init`

### 5. Beneficiary Manipulation

**Risk**: Owner sets self as beneficiary to bypass release logic

**Mitigations**:
- `create_vault` rejects owner == beneficiary
- `set_beneficiaries` rejects owner in beneficiary list
- Returns `ContractError::InvalidBeneficiary`

### 6. Withdrawal Without Second Factor

**Risk**: Owner key compromise leads to unauthorized withdrawal even when the owner has opted into 2FA

**Mitigations**:
- If a vault has 2FA enabled (`enable_2fa`), `withdraw` requires `is_2fa_verified` to return `true` for the current ledger session before it proceeds
- Verification is confirmed off-chain (`confirm_2fa`, after OTP/SMS/email validation) and expires 1 hour after confirmation
- Returns `ContractError::TwoFactorRequired` if 2FA is enabled but not currently verified
- Vaults without 2FA enabled are unaffected and withdraw normally

## Security Best Practices

- All owner actions require `owner.require_auth()`
- Structured error handling via ContractError enum
- Comprehensive test coverage for edge cases
- State validation before mutations
- TTL extension on all storage operations

## Audit Status

Not yet audited. Community review welcome.
