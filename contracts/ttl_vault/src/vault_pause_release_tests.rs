#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token::StellarAssetClient,
    Address, Env,
};

fn setup() -> (Env, Address, Address, TtlVaultContractClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();

    let owner = Address::generate(&env);
    let beneficiary = Address::generate(&env);
    let admin = Address::generate(&env);

    let token_admin = Address::generate(&env);
    let token_address = env
        .register_stellar_asset_contract_v2(token_admin)
        .address();
    StellarAssetClient::new(&env, &token_address).mint(&owner, &1_000_000);

    let contract_address = env.register_contract(None, TtlVaultContract);
    let client = TtlVaultContractClient::new(&env, &contract_address);
    client.initialize(&token_address, &admin);

    let client: TtlVaultContractClient<'static> = unsafe { core::mem::transmute(client) };
    (env, owner, beneficiary, client)
}

#[test]
#[should_panic(expected = "VaultPaused")]
fn test_trigger_release_blocked_when_vault_paused() {
    let (env, owner, beneficiary, client) = setup();
    let interval = MIN_CHECK_IN_INTERVAL;
    let vault_id = client.create_vault(&owner, &beneficiary, &interval, &None);
    client.deposit(&vault_id, &owner, &100_000i128);

    client.pause_vault(&vault_id, &owner);

    // Expire the vault's TTL
    let expired_at = env.ledger().timestamp() + interval + 1;
    env.ledger().set_timestamp(expired_at);

    // Vault-level pause must block release even though the contract-wide
    // pause flag is untouched and the vault has expired.
    client.trigger_release(&vault_id);
}

#[test]
fn test_trigger_release_succeeds_after_resume() {
    let (env, owner, beneficiary, client) = setup();
    let interval = MIN_CHECK_IN_INTERVAL;
    let vault_id = client.create_vault(&owner, &beneficiary, &interval, &None);
    client.deposit(&vault_id, &owner, &100_000i128);

    client.pause_vault(&vault_id, &owner);
    client.resume_vault(&vault_id, &owner);

    let expired_at = env.ledger().timestamp() + interval + 1;
    env.ledger().set_timestamp(expired_at);

    client.trigger_release(&vault_id);
    assert_eq!(client.get_vault(&vault_id).status, ReleaseStatus::Released);
}
