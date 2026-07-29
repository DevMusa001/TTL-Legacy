#!/usr/bin/env python3
"""
Generate seed corpus files for fuzz testing.
These seed files provide structured starting points for libfuzzer.
"""

import struct
import os
from pathlib import Path

def create_corpus_dirs():
    """Create corpus directories if they don't exist"""
    corpus_dirs = [
        "corpus/fuzz_create_vault",
        "corpus/fuzz_deposit",
        "corpus/fuzz_withdraw",
        "corpus/fuzz_check_in",
    ]
    for d in corpus_dirs:
        Path(d).mkdir(parents=True, exist_ok=True)

def generate_create_vault_corpus():
    """Generate seed inputs for create_vault fuzz target"""
    corpus_dir = Path("corpus/fuzz_create_vault")
    
    seeds = [
        # Valid: different owner and beneficiary, reasonable interval
        {
            "name": "valid_basic",
            "owner": b'\x00' * 32,
            "beneficiary": b'\x01' * 32,
            "interval": struct.pack('<Q', 86400),  # 1 day
            "has_token": 0,
        },
        # Invalid: zero interval
        {
            "name": "invalid_zero_interval",
            "owner": b'\x00' * 32,
            "beneficiary": b'\x01' * 32,
            "interval": struct.pack('<Q', 0),
            "has_token": 0,
        },
        # Invalid: owner equals beneficiary
        {
            "name": "invalid_same_owner_beneficiary",
            "owner": b'\x00' * 32,
            "beneficiary": b'\x00' * 32,
            "interval": struct.pack('<Q', 86400),
            "has_token": 0,
        },
        # Edge case: very long interval
        {
            "name": "edge_max_interval",
            "owner": b'\x00' * 32,
            "beneficiary": b'\x01' * 32,
            "interval": struct.pack('<Q', 0xFFFFFFFFFFFFFFFF),  # u64::MAX
            "has_token": 0,
        },
        # Valid: with custom token
        {
            "name": "valid_with_token",
            "owner": b'\x00' * 32,
            "beneficiary": b'\x01' * 32,
            "interval": struct.pack('<Q', 604800),  # 1 week
            "has_token": 1,
            "token": b'\x02' * 32,
        },
        # Boundary: minimum valid interval (1 second)
        {
            "name": "boundary_min_interval",
            "owner": b'\x00' * 32,
            "beneficiary": b'\x01' * 32,
            "interval": struct.pack('<Q', 1),
            "has_token": 0,
        },
    ]
    
    for seed in seeds:
        data = (
            seed["owner"] +
            seed["beneficiary"] +
            seed["interval"] +
            struct.pack('B', seed["has_token"])
        )
        if seed.get("token"):
            data += seed["token"]
        
        filepath = corpus_dir / seed["name"]
        filepath.write_bytes(data)
    
    print(f"✓ Created {len(seeds)} create_vault corpus seeds")

def generate_deposit_corpus():
    """Generate seed inputs for deposit fuzz target"""
    corpus_dir = Path("corpus/fuzz_deposit")
    
    seeds = [
        # Valid: simple deposit
        {
            "name": "valid_basic",
            "vault_id": struct.pack('<Q', 1),
            "depositor": b'\x00' * 32,
            "amount": struct.pack('<q', 1000),
            "current_balance": struct.pack('<q', 0),
            "max_deposit": struct.pack('<q', -1),  # i128 max (positive)
        },
        # Invalid: zero amount
        {
            "name": "invalid_zero_amount",
            "vault_id": struct.pack('<Q', 1),
            "depositor": b'\x00' * 32,
            "amount": struct.pack('<q', 0),
            "current_balance": struct.pack('<q', 0),
            "max_deposit": struct.pack('<q', -1),
        },
        # Invalid: negative amount
        {
            "name": "invalid_negative_amount",
            "vault_id": struct.pack('<Q', 1),
            "depositor": b'\x00' * 32,
            "amount": struct.pack('<q', -1000),
            "current_balance": struct.pack('<q', 0),
            "max_deposit": struct.pack('<q', -1),
        },
        # Edge case: balance overflow (approaching i128::MAX)
        {
            "name": "edge_balance_overflow",
            "vault_id": struct.pack('<Q', 1),
            "depositor": b'\x00' * 32,
            "amount": struct.pack('<q', 9223372036854775807),  # i64::MAX
            "current_balance": struct.pack('<q', 9223372036854775806),
            "max_deposit": struct.pack('<q', -1),
        },
        # Edge case: deposit limit exceeded
        {
            "name": "edge_deposit_limit_exceeded",
            "vault_id": struct.pack('<Q', 1),
            "depositor": b'\x00' * 32,
            "amount": struct.pack('<q', 1000),
            "current_balance": struct.pack('<q', 5000),
            "max_deposit": struct.pack('<q', 5000),
        },
        # Large deposit
        {
            "name": "large_deposit",
            "vault_id": struct.pack('<Q', 999),
            "depositor": b'\xFF' * 32,
            "amount": struct.pack('<q', 1000000000),
            "current_balance": struct.pack('<q', 1000000000),
            "max_deposit": struct.pack('<q', -1),
        },
    ]
    
    for seed in seeds:
        data = (
            seed["vault_id"] +
            seed["depositor"] +
            seed["amount"] +
            seed["current_balance"] +
            seed["max_deposit"]
        )
        filepath = corpus_dir / seed["name"]
        filepath.write_bytes(data)
    
    print(f"✓ Created {len(seeds)} deposit corpus seeds")

def generate_withdraw_corpus():
    """Generate seed inputs for withdraw fuzz target"""
    corpus_dir = Path("corpus/fuzz_withdraw")
    
    seeds = [
        # Valid: simple withdrawal
        {
            "name": "valid_basic",
            "vault_id": struct.pack('<Q', 1),
            "caller": b'\x00' * 32,
            "amount": struct.pack('<q', 100),
            "vault_balance": struct.pack('<q', 1000),
            "withdrawal_threshold": struct.pack('<q', -1),
            "min_balance_guard": struct.pack('<q', 0),
        },
        # Invalid: zero amount
        {
            "name": "invalid_zero_amount",
            "vault_id": struct.pack('<Q', 1),
            "caller": b'\x00' * 32,
            "amount": struct.pack('<q', 0),
            "vault_balance": struct.pack('<q', 1000),
            "withdrawal_threshold": struct.pack('<q', -1),
            "min_balance_guard": struct.pack('<q', 0),
        },
        # Invalid: insufficient balance
        {
            "name": "invalid_insufficient_balance",
            "vault_id": struct.pack('<Q', 1),
            "caller": b'\x00' * 32,
            "amount": struct.pack('<q', 2000),
            "vault_balance": struct.pack('<q', 1000),
            "withdrawal_threshold": struct.pack('<q', -1),
            "min_balance_guard": struct.pack('<q', 0),
        },
        # Edge case: withdrawal approval threshold
        {
            "name": "edge_withdrawal_threshold_exceeded",
            "vault_id": struct.pack('<Q', 1),
            "caller": b'\x00' * 32,
            "amount": struct.pack('<q', 1000),
            "vault_balance": struct.pack('<q', 10000),
            "withdrawal_threshold": struct.pack('<q', 500),
            "min_balance_guard": struct.pack('<q', 0),
        },
        # Edge case: minimum balance guard
        {
            "name": "edge_minimum_balance_guard",
            "vault_id": struct.pack('<Q', 1),
            "caller": b'\x00' * 32,
            "amount": struct.pack('<q', 6000),
            "vault_balance": struct.pack('<q', 10000),
            "withdrawal_threshold": struct.pack('<q', -1),
            "min_balance_guard": struct.pack('<q', 5000),
        },
        # Boundary: exact withdrawal to minimum
        {
            "name": "boundary_exact_minimum_balance",
            "vault_id": struct.pack('<Q', 1),
            "caller": b'\x00' * 32,
            "amount": struct.pack('<q', 5000),
            "vault_balance": struct.pack('<q', 10000),
            "withdrawal_threshold": struct.pack('<q', -1),
            "min_balance_guard": struct.pack('<q', 5000),
        },
    ]
    
    for seed in seeds:
        data = (
            seed["vault_id"] +
            seed["caller"] +
            seed["amount"] +
            seed["vault_balance"] +
            seed["withdrawal_threshold"] +
            seed["min_balance_guard"]
        )
        filepath = corpus_dir / seed["name"]
        filepath.write_bytes(data)
    
    print(f"✓ Created {len(seeds)} withdraw corpus seeds")

def generate_check_in_corpus():
    """Generate seed inputs for check_in fuzz target"""
    corpus_dir = Path("corpus/fuzz_check_in")
    
    seeds = [
        # Valid: on-time check-in
        {
            "name": "valid_on_time",
            "vault_id": struct.pack('<Q', 1),
            "caller": b'\x00' * 32,
            "passkey_hash": b'\x00' * 32,
            "nonce": struct.pack('<Q', 0),
            "current_time": struct.pack('<Q', 1000),
            "last_check_in": struct.pack('<Q', 100),
            "interval": struct.pack('<Q', 86400),
            "balance": struct.pack('<q', 10000),
            "penalty_bps": struct.pack('<H', 0),
        },
        # Invalid: zero interval
        {
            "name": "invalid_zero_interval",
            "vault_id": struct.pack('<Q', 1),
            "caller": b'\x00' * 32,
            "passkey_hash": b'\x00' * 32,
            "nonce": struct.pack('<Q', 0),
            "current_time": struct.pack('<Q', 1000),
            "last_check_in": struct.pack('<Q', 100),
            "interval": struct.pack('<Q', 0),
            "balance": struct.pack('<q', 10000),
            "penalty_bps": struct.pack('<H', 0),
        },
        # Invalid: vault expired (current_time >= last_check_in + interval)
        {
            "name": "invalid_vault_expired",
            "vault_id": struct.pack('<Q', 1),
            "caller": b'\x00' * 32,
            "passkey_hash": b'\x00' * 32,
            "nonce": struct.pack('<Q', 0),
            "current_time": struct.pack('<Q', 100000),
            "last_check_in": struct.pack('<Q', 100),
            "interval": struct.pack('<Q', 1000),
            "balance": struct.pack('<q', 10000),
            "penalty_bps": struct.pack('<H', 0),
        },
        # Edge case: TTL cap exceeded
        {
            "name": "edge_ttl_cap_exceeded",
            "vault_id": struct.pack('<Q', 1),
            "caller": b'\x00' * 32,
            "passkey_hash": b'\x00' * 32,
            "nonce": struct.pack('<Q', 0),
            "current_time": struct.pack('<Q', 1000),
            "last_check_in": struct.pack('<Q', 100),
            "interval": struct.pack('<Q', 0xFFFFFFFFFFFFFFFF),  # Very large interval
            "balance": struct.pack('<q', 10000),
            "penalty_bps": struct.pack('<H', 0),
        },
        # Edge case: inactivity penalty calculation
        {
            "name": "edge_inactivity_penalty",
            "vault_id": struct.pack('<Q', 1),
            "caller": b'\x00' * 32,
            "passkey_hash": b'\x00' * 32,
            "nonce": struct.pack('<Q', 0),
            "current_time": struct.pack('<Q', 1000000),
            "last_check_in": struct.pack('<Q', 100),
            "interval": struct.pack('<Q', 86400),
            "balance": struct.pack('<q', 10000),
            "penalty_bps": struct.pack('<H', 500),  # 5% penalty
        },
        # Boundary: check-in just before expiry
        {
            "name": "boundary_just_before_expiry",
            "vault_id": struct.pack('<Q', 1),
            "caller": b'\x00' * 32,
            "passkey_hash": b'\x00' * 32,
            "nonce": struct.pack('<Q', 0),
            "current_time": struct.pack('<Q', 86499),  # Just before expiry
            "last_check_in": struct.pack('<Q', 100),
            "interval": struct.pack('<Q', 86400),
            "balance": struct.pack('<q', 10000),
            "penalty_bps": struct.pack('<H', 0),
        },
    ]
    
    for seed in seeds:
        data = (
            seed["vault_id"] +
            seed["caller"] +
            seed["passkey_hash"] +
            seed["nonce"] +
            seed["current_time"] +
            seed["last_check_in"] +
            seed["interval"] +
            seed["balance"] +
            seed["penalty_bps"]
        )
        filepath = corpus_dir / seed["name"]
        filepath.write_bytes(data)
    
    print(f"✓ Created {len(seeds)} check_in corpus seeds")

if __name__ == "__main__":
    create_corpus_dirs()
    generate_create_vault_corpus()
    generate_deposit_corpus()
    generate_withdraw_corpus()
    generate_check_in_corpus()
    print("\n✓ All corpus files generated successfully!")
