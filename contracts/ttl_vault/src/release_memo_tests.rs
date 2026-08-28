#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, token::StellarAssetClient, Address, Bytes, Env};

fn setup_release_memo_test() -> (Env, Address, Address, TtlVaultContractClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();

    let owner = Address::generate(&env);
    let beneficiary = Address::generate(&env);
    let admin = Address::generate(&env);

    let token_admin = Address::generate(&env);
    let token_address = env
        .register_stellar_asset_contract_v2(token_admin)
        .address();
    StellarAssetClient::new(&env, &token_address).mint(&owner, &10_000_000);

    let contract_address = env.register_contract(None, TtlVaultContract);
    let client = TtlVaultContractClient::new(&env, &contract_address);
    client.initialize(&token_address, &admin);

    let client: TtlVaultContractClient<'static> = unsafe { core::mem::transmute(client) };
    (env, owner, beneficiary, client)
}

#[test]
fn test_set_release_memo_succeeds() {
    let (env, owner, beneficiary, client) = setup_release_memo_test();
    let vault_id = client.create_vault(&owner, &beneficiary, &1_000u64, &None);

    let memo = Bytes::from_slice(&env, b"For my family");
    client.set_release_memo(&vault_id, &owner, &memo);

    assert_eq!(client.get_release_memo(&vault_id), memo);
}

#[test]
fn test_update_release_memo_overwrites_previous() {
    let (env, owner, beneficiary, client) = setup_release_memo_test();
    let vault_id = client.create_vault(&owner, &beneficiary, &1_000u64, &None);

    let first = Bytes::from_slice(&env, b"Old memo");
    let second = Bytes::from_slice(&env, b"New memo");
    client.set_release_memo(&vault_id, &owner, &first);
    client.set_release_memo(&vault_id, &owner, &second);

    assert_eq!(client.get_release_memo(&vault_id), second);
}

#[test]
fn test_release_memo_defaults_to_empty() {
    let (_env, owner, beneficiary, client) = setup_release_memo_test();
    let vault_id = client.create_vault(&owner, &beneficiary, &1_000u64, &None);

    assert_eq!(client.get_release_memo(&vault_id).len(), 0);
}

#[test]
fn test_set_release_memo_nil_clears_memo() {
    let (env, owner, beneficiary, client) = setup_release_memo_test();
    let vault_id = client.create_vault(&owner, &beneficiary, &1_000u64, &None);

    client.set_release_memo(&vault_id, &owner, &Bytes::from_slice(&env, b"Something"));
    client.set_release_memo(&vault_id, &owner, &Bytes::new(&env));

    assert_eq!(client.get_release_memo(&vault_id).len(), 0);
}

#[test]
#[should_panic(expected = "NotOwner")]
fn test_set_release_memo_requires_owner() {
    let (env, owner, beneficiary, client) = setup_release_memo_test();
    let vault_id = client.create_vault(&owner, &beneficiary, &1_000u64, &None);
    let stranger = Address::generate(&env);

    client.set_release_memo(&vault_id, &stranger, &Bytes::from_slice(&env, b"Nope"));
}

#[test]
#[should_panic(expected = "InvalidAmount")]
fn test_set_release_memo_rejects_oversized_memo() {
    let (env, owner, beneficiary, client) = setup_release_memo_test();
    let vault_id = client.create_vault(&owner, &beneficiary, &1_000u64, &None);

    let too_long = Bytes::from_slice(&env, &[0u8; 257]);
    client.set_release_memo(&vault_id, &owner, &too_long);
}
