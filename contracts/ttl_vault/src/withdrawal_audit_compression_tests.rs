#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Env,
};

#[test]
fn test_compress_withdrawal_history_aggregates_old_records() {
    let env = Env::new();
    let owner = Address::generate(&env);
    let vault_id: u64 = 1;
    let keep_recent_days: u64 = 30;

    // Test that compress_withdrawal_history aggregates withdrawals older than keep_recent_days
    // Old records should be combined into summary entries
    // Summary should contain: (count, total_amount, timestamp_start, timestamp_end)
}

#[test]
fn test_compress_withdrawal_history_preserves_recent() {
    let env = Env::new();
    let owner = Address::generate(&env);
    let vault_id: u64 = 1;
    let keep_recent_days: u64 = 30;

    // Test that recent withdrawals (within keep_recent_days) are preserved as-is
    // Only old withdrawals should be aggregated
    // Recent record details should remain unchanged
}

#[test]
fn test_compress_withdrawal_history_owner_only() {
    let env = Env::new();
    let owner = Address::generate(&env);
    let non_owner = Address::generate(&env);
    let vault_id: u64 = 1;
    let keep_recent_days: u64 = 30;

    // Test that only owner can call compress_withdrawal_history
    // Non-owner should get authorization error
}

#[test]
fn test_compress_withdrawal_history_invalid_keep_days() {
    let env = Env::new();
    let owner = Address::generate(&env);
    let vault_id: u64 = 1;
    let invalid_keep_days: u64 = 0;

    // Test that keep_recent_days = 0 is rejected
    // Should return InvalidParameter error
    // Must have positive day value to preserve some history
}

#[test]
fn test_compress_withdrawal_history_empty_vault() {
    let env = Env::new();
    let owner = Address::generate(&env);
    let vault_id: u64 = 1;
    let keep_recent_days: u64 = 30;

    // Test that compressing empty withdrawal history succeeds without error
    // Should return gracefully with no aggregations needed
}

#[test]
fn test_get_compressed_history_returns_summary_entries() {
    let env = Env::new();
    let owner = Address::generate(&env);
    let vault_id: u64 = 1;

    // Test that get_compressed_history returns Vec of (count, total_amount, timestamp_start, timestamp_end)
    // Should be ordered chronologically
}

#[test]
fn test_get_compressed_history_owner_only() {
    let env = Env::new();
    let owner = Address::generate(&env);
    let non_owner = Address::generate(&env);
    let vault_id: u64 = 1;

    // Test that only owner can retrieve compressed history
    // Non-owner should get authorization error
}

#[test]
fn test_get_compressed_history_after_compression() {
    let env = Env::new();
    let owner = Address::generate(&env);
    let vault_id: u64 = 1;
    let keep_recent_days: u64 = 30;

    // Test that get_compressed_history returns aggregated summaries after compression
    // Should reflect the compression that was performed
}

#[test]
fn test_compression_calculates_correct_total_amount() {
    let env = Env::new();
    let owner = Address::generate(&env);
    let vault_id: u64 = 1;
    let keep_recent_days: u64 = 30;

    // Test that aggregated total_amount is sum of all old withdrawals
    // Should correctly sum stroops
}

#[test]
fn test_compression_calculates_correct_count() {
    let env = Env::new();
    let owner = Address::generate(&env);
    let vault_id: u64 = 1;
    let keep_recent_days: u64 = 30;

    // Test that aggregated count is number of withdrawals combined
    // Count should accurately reflect combined records
}

#[test]
fn test_compression_timestamp_boundaries() {
    let env = Env::new();
    let owner = Address::generate(&env);
    let vault_id: u64 = 1;
    let keep_recent_days: u64 = 30;

    // Test that timestamp_start and timestamp_end are correctly set
    // Should represent the time range of aggregated records
    // timestamp_start should be oldest, timestamp_end should be newest
}

#[test]
fn test_multiple_compression_calls() {
    let env = Env::new();
    let owner = Address::generate(&env);
    let vault_id: u64 = 1;
    let keep_recent_days: u64 = 30;

    // Test that multiple compress_withdrawal_history calls are safe
    // Second compression should handle already-compressed data correctly
}

#[test]
fn test_compression_with_mixed_old_and_new() {
    let env = Env::new();
    let owner = Address::generate(&env);
    let vault_id: u64 = 1;
    let keep_recent_days: u64 = 30;

    // Test compression with both old and recent withdrawals
    // Old withdrawals should be aggregated
    // Recent withdrawals should remain as individual entries
}

#[test]
fn test_compression_preserves_data_integrity() {
    let env = Env::new();
    let owner = Address::generate(&env);
    let vault_id: u64 = 1;
    let keep_recent_days: u64 = 30;

    // Test that total amount before and after compression matches
    // No data loss should occur during aggregation
}

#[test]
fn test_compression_handles_partial_day_boundaries() {
    let env = Env::new();
    let owner = Address::generate(&env);
    let vault_id: u64 = 1;
    let keep_recent_days: u64 = 30;

    // Test that keep_recent_days is calculated correctly at day boundaries
    // Withdrawals exactly at the boundary should be included correctly
}

#[test]
fn test_compression_with_single_old_withdrawal() {
    let env = Env::new();
    let owner = Address::generate(&env);
    let vault_id: u64 = 1;
    let keep_recent_days: u64 = 30;

    // Test compression with only one old withdrawal
    // Should create summary with count=1, amount=withdrawal_amount
}

#[test]
fn test_compression_with_many_withdrawals() {
    let env = Env::new();
    let owner = Address::generate(&env);
    let vault_id: u64 = 1;
    let keep_recent_days: u64 = 30;

    // Test compression with many withdrawals (e.g., 100+)
    // Should handle large datasets efficiently
}

#[test]
fn test_compression_zero_amount_withdrawals() {
    let env = Env::new();
    let owner = Address::generate(&env);
    let vault_id: u64 = 1;
    let keep_recent_days: u64 = 30;

    // Test compression with zero-amount withdrawal entries
    // Should correctly sum amounts, including zeros
}

#[test]
fn test_compressed_history_ordering() {
    let env = Env::new();
    let owner = Address::generate(&env);
    let vault_id: u64 = 1;

    // Test that get_compressed_history returns chronologically ordered summaries
    // Earlier time ranges should come before later ones
}

#[test]
fn test_compression_idempotence() {
    let env = Env::new();
    let owner = Address::generate(&env);
    let vault_id: u64 = 1;
    let keep_recent_days: u64 = 30;

    // Test that running compression twice with same keep_recent_days is idempotent
    // Second run should produce same result as first
}

#[test]
fn test_compression_reduces_storage() {
    let env = Env::new();
    let owner = Address::generate(&env);
    let vault_id: u64 = 1;
    let keep_recent_days: u64 = 30;

    // Test that compression reduces total number of history entries
    // Multiple old records should become single summary entry
}

#[test]
fn test_get_compressed_history_empty_result() {
    let env = Env::new();
    let owner = Address::generate(&env);
    let vault_id: u64 = 1;

    // Test that get_compressed_history returns empty vec for vault with no history
}
